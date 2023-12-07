use crate::{
    contract_actions::restaking_base_callbacks::ext_restaking_base_callbacks,
    ext_contracts::ext_restaking_base, *,
};
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
        let mut validator_set = self
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
        match slach_packet_data.infraction.as_str() {
            "INFRACTION_DOWNTIME" => validator_set.jail_validator(&validator_id),
            "INFRACTION_DOUBLE_SIGN" => {
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
            _ => (),
        }
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
        //
        let anchor_settings = self.anchor_settings.get().unwrap();
        let mut reward_distribution = RewardDistribution {
            validator_set_id,
            amount: anchor_settings.era_reward,
            timestamp: env::block_timestamp(),
        };
        self.pending_rewards.append(&mut reward_distribution);
        log!(
            "Reward distribution request from `near-ibc` is recorded: {:?}",
            reward_distribution
        );
    }
}
