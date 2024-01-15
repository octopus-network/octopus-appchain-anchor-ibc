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
use near_sdk::NearToken;

/// Any account can call these functions.
pub trait PermissionlessActions {
    /// Fetch validator set from restaking base contract.
    fn fetch_validator_set_from_restaking_base(&mut self);
    /// Send VSC packet to appchain via near-ibc contract.
    fn send_vsc_packet_to_appchain(&mut self);
    /// Distribute pending rewards to validators.
    fn distribute_pending_rewards(&mut self) -> ProcessingResult;
    /// Unjail the given validator.
    fn unjail_validator(&mut self, validator_id: AccountId);
    /// Process the first pending slash packet.
    fn process_first_pending_slash_packet(&mut self);
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
        if let Some(validator_set) = self.validator_set_histories.get_last() {
            self.send_vsc_packet(
                &validator_set,
                &self.validator_set_histories.get_second_last(),
                vec![],
                vec![],
            );
        }
    }
    //
    fn distribute_pending_rewards(&mut self) -> ProcessingResult {
        assert!(
            env::prepaid_gas() >= Gas::from_tgas(T_GAS_FOR_SIMPLE_FUNCTION_CALL * 10),
            "Not enough gas, needs at least {}T.",
            T_GAS_FOR_SIMPLE_FUNCTION_CALL * 10
        );
        let (index, reward_distribution) = self
            .get_first_undistributed_reward()
            .expect("No pending reward.");
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
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(Gas::from_tgas(T_GAS_FOR_SIMPLE_FUNCTION_CALL * 8))
            .with_unused_gas_weight(0)
            .ft_transfer_call(
                self.lpos_market_contract.clone(),
                reward_distribution.amount,
                None,
                near_sdk::serde_json::to_string(&msg).unwrap(),
            )
            .then(
                ext_reward_token_callbacks::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(T_GAS_FOR_SIMPLE_FUNCTION_CALL * 8))
                    .with_unused_gas_weight(0)
                    .ft_transfer_call_callback(msg, reward_distribution, U64::from(index)),
            );
        ProcessingResult::NeedMoreGas
    }
    //
    fn unjail_validator(&mut self, validator_id: AccountId) {
        let mut validator_set = self
            .validator_set_histories
            .get_last()
            .expect("No validator set exists.");
        assert!(
            validator_set.contains_validator(&validator_id),
            "Validator {:?} is not in the latest validator set.",
            validator_id
        );
        let anchor_settings = self.anchor_settings.get().unwrap();
        validator_set.unjail_validator(&validator_id, anchor_settings.min_unjail_interval.0);
        self.validator_set_histories.update_last(&validator_set);
        self.send_vsc_packet(
            &validator_set,
            &self.validator_set_histories.get_second_last(),
            vec![],
            vec![],
        );
    }
    //
    fn process_first_pending_slash_packet(&mut self) {
        if let Some(packet_string) = self.pending_slash_packets.get_first() {
            self.internal_process_slash_packet(
                &near_sdk::serde_json::from_str::<SlashPacketView>(packet_string.as_str())
                    .expect("Invalid slash packet data."),
            );
            self.pending_slash_packets
                .remove_first(env::prepaid_gas().saturating_sub(env::used_gas()));
            log!("The first slash packet has been applied: {}", packet_string);
        } else {
            log!("No pending slash packet.");
        }
    }
}

impl AppchainAnchor {
    pub fn send_vsc_packet(
        &mut self,
        validator_set: &ValidatorSet,
        previous_validator_set: &Option<ValidatorSet>,
        removing_pubkeys: Vec<Vec<u8>>,
        slash_acks: Vec<String>,
    ) {
        assert!(
            self.appchain_state == AppchainState::Active,
            "The state of appchain must be 'Active'."
        );
        ext_near_ibc::ext(self.near_ibc_contract.clone()).send_vsc_packet(
            self.get_chain_id(),
            self.generate_vsc_packet_data(
                validator_set,
                previous_validator_set,
                &removing_pubkeys,
                &slash_acks,
            ),
            self.anchor_settings
                .get()
                .unwrap()
                .vsc_packet_timeout_interval,
        );
    }
    //
    fn generate_vsc_packet_data(
        &self,
        validator_set: &ValidatorSet,
        previous_validator_set: &Option<ValidatorSet>,
        removing_pubkeys: &Vec<Vec<u8>>,
        slash_acks: &Vec<String>,
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
        let mut validator_pubkeys = match previous_validator_set {
            Some(previous_validator_set) => {
                let mut previous_vs_pubkeys = previous_validator_set
                    .active_validators()
                    .iter()
                    .map(|(validator_id, stake)| ValidatorKeyAndPower {
                        public_key: self.validator_id_to_pubkey_map.get(&validator_id).unwrap(),
                        power: U64::from((stake.0 / NEAR_SCALE) as u64),
                    })
                    .filter(|vkp| vkp.power.0 > 0)
                    .collect::<Vec<ValidatorKeyAndPower>>();
                for pvkp in &mut previous_vs_pubkeys {
                    if let Some(vkp) = vs_pubkeys
                        .iter()
                        .find(|vkp| vkp.public_key == pvkp.public_key)
                    {
                        pvkp.power = vkp.power;
                    } else {
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
        for pubkey in removing_pubkeys {
            validator_pubkeys.push(ValidatorKeyAndPower {
                public_key: pubkey.clone(),
                power: U64::from(0),
            });
        }
        VscPacketData {
            validator_pubkeys,
            validator_set_id: U64::from(validator_set.id()),
            slash_acks: slash_acks.clone(),
        }
    }
    //
    fn get_first_undistributed_reward(&self) -> Option<(u64, RewardDistribution)> {
        let index_range = self.pending_rewards.index_range();
        for index in index_range.start_index.0..index_range.end_index.0 + 1 {
            let reward_distribution = self.pending_rewards.get(&index).unwrap();
            if !reward_distribution.distributed {
                return Some((index, reward_distribution.clone()));
            }
        }
        None
    }
    ///
    pub fn internal_process_slash_packet(&mut self, slash_packet_view: &SlashPacketView) {
        let mut validator_set = self
            .validator_set_histories
            .get_last()
            .expect("No validator set exists, should not happen.");
        let slashing_validator = slash_packet_view.clone().validator.expect(
            format!(
                "Validator is empty, invalid slash packet: {}",
                near_sdk::serde_json::to_string(slash_packet_view).unwrap()
            )
            .as_str(),
        );
        let validator_id = self
            .validator_address_to_id_map
            .get(&slashing_validator.address)
            .expect(
                format!(
                    "Validator address {:?} is not registered.",
                    slashing_validator.address
                )
                .as_str(),
            );
        let removing_pubkeys = if validator_set.id() > slash_packet_view.valset_update_id + 1 {
            emit_nep297_event("SLASH_PACKET_TOO_OLD", slash_packet_view);
            vec![]
        } else {
            self.validator_id_to_pubkey_map
                .get(&validator_id)
                .map_or(vec![], |v| vec![v])
        };
        let anchor_settings = self.anchor_settings.get().unwrap();
        match slash_packet_view.infraction.as_str() {
            "INFRACTION_DOWNTIME" => {
                validator_set.jail_validator(&validator_id);
                self.validator_set_histories.update_last(&validator_set);
                self.send_vsc_packet(
                    &validator_set,
                    &self.validator_set_histories.get_second_last(),
                    removing_pubkeys,
                    vec![calculate_bech32_address(
                        anchor_settings.appchain_address_bech32_hrp.clone(),
                        slashing_validator.address.clone(),
                    )],
                );
            }
            "INFRACTION_DOUBLE_SIGN" => {
                unimplemented!("INFRACTION_DOUBLE_SIGN");
                // let validator = validator_set.get_validator(&validator_id).expect(
                //     format!(
                //         "Validator for address {:?} is not found in validator set {}.",
                //         slashing_validator.address, slash_packet_data.valset_update_id
                //     )
                //     .as_str(),
                // );
                // let slash_items = vec![(validator.validator_id, U128::from(validator.total_stake))];
                // ext_restaking_base::ext(self.restaking_base_contract.clone())
                //     .slash_request(self.appchain_id.clone(), slash_items.clone(), String::new())
                //     .then(
                //         ext_restaking_base_callbacks::ext(env::current_account_id())
                //             .slash_request_callback(slash_items),
                //     );
            }
            _ => (),
        }
    }
}
