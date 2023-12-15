use crate::{
    anchor_viewer::AnchorViewer,
    contract_actions::{
        restaking_base_callbacks::ext_restaking_base_callbacks,
        reward_token_callbacks::ext_reward_token_callbacks,
    },
    ext_contracts::{ext_near_ibc, ext_restaking_base},
    *,
};
use near_contract_standards::fungible_token::core::ext_ft_core;

/// Any account can call these functions.
pub trait PermissionlessActions {
    /// Fetch validator set from restaking base contract.
    fn fetch_validator_set_from_restaking_base(&mut self);
    /// Send VSC packet to appchain via near-ibc contract.
    fn send_vsc_packet_to_appchain(&mut self);
    /// Distribute pending rewards to validators.
    fn distribute_pending_rewards(&mut self) -> ProcessingResult;
}

#[near_bindgen]
impl PermissionlessActions for AppchainAnchor {
    //
    fn fetch_validator_set_from_restaking_base(&mut self) {
        let anchor_settings = self.anchor_settings.get().unwrap();
        if let Some(latest_validator_set) = self.validator_set_histories.get_last() {
            assert!(
                env::block_timestamp() - latest_validator_set.timestamp()
                    > anchor_settings.min_interval_for_new_validator_set.0,
                "The interval between two validator sets is too short."
            );
        }
        let consumer_chain_id = format!("cosmos:{}", self.appchain_id);
        ext_restaking_base::ext(self.restaking_base_contract.clone())
            .get_validator_set(consumer_chain_id, anchor_settings.max_count_of_validators)
            .then(
                ext_restaking_base_callbacks::ext(env::current_account_id())
                    .get_validator_set_callback(),
            );
    }
    //
    fn send_vsc_packet_to_appchain(&mut self) {
        self.send_vsc_packet(Vec::new());
    }
    //
    fn distribute_pending_rewards(&mut self) -> ProcessingResult {
        assert!(
            env::prepaid_gas() >= Gas::ONE_TERA.mul(T_GAS_FOR_SIMPLE_FUNCTION_CALL * 10),
            "Not enough gas, needs at least {}T.",
            T_GAS_FOR_SIMPLE_FUNCTION_CALL * 10
        );
        if self.pending_rewards.len() == 0 {
            return ProcessingResult::Ok;
        }
        let reward_distribution = self.pending_rewards.get_first().unwrap();
        if self.locked_reward_token_amount < reward_distribution.amount.0 {
            return ProcessingResult::Error(
                "The locked reward token amount is not enough.".to_string(),
            );
        }
        //
        let validator_set = self
            .validator_set_histories
            .get(&reward_distribution.validator_set_id.0)
            .expect(
                format!(
                    "Invalid validator set id in pending rewards record: {}, should not happen.",
                    reward_distribution.validator_set_id.0
                )
                .as_str(),
            );
        let msg = FtTransferMessage::AnchorDepositRewardMsg(AnchorDepositRewardMsg {
            consumer_chain_id: format!("cosmos:{}", self.appchain_id.clone()),
            validator_set: validator_set.active_validators(),
            sequence: validator_set.sequence().into(),
        });
        //
        ext_ft_core::ext(self.reward_token_contract.clone())
            .with_attached_deposit(1)
            .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_SIMPLE_FUNCTION_CALL * 8))
            .ft_transfer_call(
                self.lpos_market_contract.clone(),
                reward_distribution.amount,
                None,
                near_sdk::serde_json::to_string(&msg).unwrap(),
            )
            .then(
                ext_reward_token_callbacks::ext(env::current_account_id())
                    .ft_transfer_call_callback(msg, reward_distribution.amount),
            );
        ProcessingResult::NeedMoreGas
    }
}

impl AppchainAnchor {
    pub fn send_vsc_packet(&mut self, removing_addresses: Vec<String>) {
        assert!(
            self.appchain_state == AppchainState::Active,
            "The state of appchain must be 'Active'."
        );
        if let Some(latest_validator_set) = self.validator_set_histories.get_last() {
            ext_near_ibc::ext(self.near_ibc_contract.clone()).send_vsc_packet(
                self.get_chain_id(),
                self.generate_vsc_packet_data(
                    &latest_validator_set,
                    &self.validator_set_histories.get_second_last(),
                    &removing_addresses,
                ),
                self.anchor_settings
                    .get()
                    .unwrap()
                    .vsc_packet_timeout_interval,
            );
        } else {
            panic!("No validator set to send.");
        }
    }
    //
    fn generate_vsc_packet_data(
        &self,
        validator_set: &ValidatorSet,
        previous_vs: &Option<ValidatorSet>,
        removing_addresses: &Vec<String>,
    ) -> VscPacketData {
        let vs_pubkeys = validator_set
            .active_validators()
            .iter()
            .map(|(validator_id, stake)| ValidatorKeyAndPower {
                public_key: self.validator_id_to_pubkey_map.get(&validator_id).unwrap(),
                power: U64::from((stake.0 / NEAR_SCALE) as u64),
            })
            .filter(|vkp| vkp.power.0 > 0)
            .collect::<Vec<ValidatorKeyAndPower>>();
        let mut validator_pubkeys = match previous_vs {
            Some(previous_vs) => {
                let mut previous_vs_pubkeys = previous_vs
                    .active_validators()
                    .iter()
                    .map(|(validator_id, stake)| ValidatorKeyAndPower {
                        public_key: self.validator_id_to_pubkey_map.get(&validator_id).unwrap(),
                        power: U64::from((stake.0 / NEAR_SCALE) as u64),
                    })
                    .filter(|vkp| vkp.power.0 > 0)
                    .collect::<Vec<ValidatorKeyAndPower>>();
                for pvkp in &mut previous_vs_pubkeys {
                    if vs_pubkeys
                        .iter()
                        .find(|vkp| vkp.public_key == pvkp.public_key)
                        .is_none()
                    {
                        pvkp.power = U64::from(0);
                    }
                }
                vs_pubkeys.iter().for_each(|vkp| {
                    if previous_vs_pubkeys
                        .iter()
                        .find(|pvkp| pvkp.public_key == vkp.public_key)
                        .is_none()
                    {
                        previous_vs_pubkeys.push(vkp.clone());
                    }
                });
                previous_vs_pubkeys.clone()
            }
            None => vs_pubkeys.clone(),
        };
        for address in removing_addresses {
            validator_pubkeys.push(ValidatorKeyAndPower {
                public_key: decode_ed25519_pubkey(address)
                    .expect(format!("Invalid removing address: {}", address).as_str()),
                power: U64::from(0),
            });
        }
        let slashed_addresses = validator_set
            .slash_ack_validators()
            .iter()
            .map(|id| {
                calculate_address(self.validator_id_to_pubkey_map.get(&id).unwrap().as_slice())
            })
            .collect();
        VscPacketData {
            validator_pubkeys,
            validator_set_id: U64::from(validator_set.id()),
            slash_acks: slashed_addresses,
        }
    }
}
