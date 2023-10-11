use crate::{
    contract_actions::restaking_base_callbacks::ext_restaking_base_callbacks,
    ext_contracts::ext_restaking_base, *,
};
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::json_types::U64;
use octopus_lpos::packet::consumer::SlashPacketData;

pub trait NearIbcActions {
    /// Interface for near-ibc to call when slash packet is received.
    fn slash_validator(&mut self, slach_packet_data: SlashPacketData);
    /// Interface for near-ibc to call when vsc_matured packet is received.
    fn on_vsc_matured(&mut self, validator_set_id: U64);
    /// Interface for near-ibc to call when distribute_reward packet is received.
    fn distribute_reward(&mut self, validator_set_id: U64);
}

#[near_bindgen]
impl NearIbcActions for AppchainAnchor {
    /// Interface for near-ibc to call when slash packet is received.
    fn slash_validator(&mut self, slach_packet_data: SlashPacketData) {
        self.assert_near_ibc_contract();
        let slashing_validator = slach_packet_data.validator.expect("Validator is empty.");
        let validator_set = self
            .validator_set_histories
            .get(&slach_packet_data.valset_update_id)
            .expect(
                format!(
                    "Invalid validator set id in slash packet data: {}",
                    slach_packet_data.valset_update_id
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
        let validator = validator_set.get_validator(&validator_id).expect(
            format!(
                "Validator for address {:?} is not found in validator set {}.",
                slashing_validator.address, slach_packet_data.valset_update_id
            )
            .as_str(),
        );
        let slash_items = vec![(validator.validator_id, U128::from(validator.total_stake))];
        ext_restaking_base::ext(self.restaking_base_contract.clone())
            .slash_request(self.appchain_id.clone(), slash_items.clone(), String::new())
            .then(
                ext_restaking_base_callbacks::ext(env::current_account_id())
                    .slash_request_callback(slash_items),
            );
    }
    /// Interface for near-ibc to call when vsc_matured packet is received.
    fn on_vsc_matured(&mut self, validator_set_id: U64) {
        self.assert_near_ibc_contract();
        if let Some(mut validator_set) = self.validator_set_histories.get(&validator_set_id.0) {
            validator_set.set_matured();
            self.validator_set_histories
                .update(&validator_set_id.0, &validator_set);
        }
    }
    /// Interface for near-ibc to call when distribute_reward packet is received.
    fn distribute_reward(&mut self, validator_set_id: U64) {
        self.assert_near_ibc_contract();
        let validator_set = self
            .validator_set_histories
            .get(&validator_set_id.0)
            .expect(format!("Invalid validator set id: {}", validator_set_id.0).as_str());
        let anchor_settings = self.anchor_settings.get().unwrap();
        //
        let msg = AnchorDepositRewardMsg {
            consumer_chain_id: self.appchain_id.clone(),
            validator_set: validator_set.active_validators(),
            sequence: validator_set.sequence().into(),
        };
        if self.locked_reward_token_amount >= anchor_settings.era_reward.0 {
            ext_ft_core::ext(self.reward_token_contract.clone()).ft_transfer_call(
                self.lpos_market_contract.clone(),
                anchor_settings.era_reward,
                None,
                near_sdk::serde_json::to_string(&msg).unwrap(),
            );
            self.locked_reward_token_amount -= anchor_settings.era_reward.0;
        } else {
            let mut pending_rewards = self.pending_rewards.get().unwrap_or_default();
            pending_rewards.push_back(RewardDistribution {
                transfer_call_msg: msg,
                amount: anchor_settings.era_reward,
            });
            self.pending_rewards.set(&pending_rewards);
            log!(
                "The locked reward token amount is not enough to distribute rewards for validator set {}.",
                validator_set_id.0
            );
        }
    }
}
