use crate::{
    contract_actions::restaking_base_callbacks::ext_restaking_base_callbacks,
    ext_contracts::{ext_near_ibc, ext_restaking_base},
    *,
};
use near_contract_standards::fungible_token::core::ext_ft_core;

const NEAR_SCALE: u128 = 1_000_000_000_000_000_000_000_000;

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
        if let Some(latest_validator_set) = self.validator_set_histories.get_latest() {
            assert!(
                latest_validator_set.matured_in_appchain()
                    && env::block_timestamp() - latest_validator_set.timestamp()
                        > anchor_settings.min_interval_for_new_validator_set.0,
                "The interval between two validator sets is too short."
            );
        }
        ext_restaking_base::ext(self.restaking_base_contract.clone())
            .get_validator_set(
                self.appchain_id.clone(),
                anchor_settings.max_count_of_validators,
            )
            .then(
                ext_restaking_base_callbacks::ext(env::current_account_id())
                    .get_validator_set_callback(),
            );
    }
    //
    fn send_vsc_packet_to_appchain(&mut self) {
        if let Some(latest_validator_set) = self.validator_set_histories.get_latest() {
            assert!(
                !latest_validator_set.matured_in_appchain(),
                "No validator set to send."
            );
            ext_near_ibc::ext(self.near_ibc_contract.clone())
                .send_vsc_packet(self.generate_vsc_packet_data(&latest_validator_set));
        } else {
            panic!("No validator set to send.");
        }
    }
    //
    fn distribute_pending_rewards(&mut self) -> ProcessingResult {
        let mut pending_rewards = self.pending_rewards.get().unwrap_or_default();
        if pending_rewards.is_empty() {
            return ProcessingResult::Ok;
        }
        let reward_distribution = pending_rewards.pop_front().unwrap();
        if self.locked_reward_token_amount < reward_distribution.amount.0 {
            return ProcessingResult::Error(
                "The locked reward token amount is not enough.".to_string(),
            );
        }
        //
        ext_ft_core::ext(self.reward_token_contract.clone()).ft_transfer_call(
            self.lpos_market_contract.clone(),
            reward_distribution.amount,
            None,
            near_sdk::serde_json::to_string(&reward_distribution.transfer_call_msg).unwrap(),
        );
        self.locked_reward_token_amount -= reward_distribution.amount.0;
        if pending_rewards.is_empty() {
            self.pending_rewards.remove();
            ProcessingResult::Ok
        } else {
            self.pending_rewards.set(&pending_rewards);
            ProcessingResult::NeedMoreGas
        }
    }
}

impl AppchainAnchor {
    fn generate_vsc_packet_data(&self, validator_set: &ValidatorSet) -> VscPacketData {
        let validator_pubkeys = validator_set
            .active_validators()
            .iter()
            .map(|(validator_id, stake)| ValidatorKeyAndPower {
                public_key: self.validator_id_to_pubkey_map.get(&validator_id).unwrap(),
                power: U64::from((stake.0 / NEAR_SCALE) as u64),
            })
            .filter(|vkp| vkp.power.0 > 0)
            .collect();
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
