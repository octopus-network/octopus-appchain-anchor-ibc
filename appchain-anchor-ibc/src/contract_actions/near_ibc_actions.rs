use crate::*;
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
        let mut packet_string = serde_json::to_string(&slach_packet_data).unwrap();
        self.pending_slash_packets.append(&mut packet_string);
        log!(
            r#"EVENT_JSON:{{"standard":"nep297","version":"1.0.0","event":"SLASH_PACKET_RECEIVED","timestamp":"{}","packet_string":"{}"}}"#,
            env::block_timestamp(),
            packet_string,
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
