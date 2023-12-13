use crate::*;
use core::convert::From;

pub trait AnchorSettingsManager {
    ///
    fn change_chain_revision_number(&mut self, value: U64);
    ///
    fn change_era_reward(&mut self, era_reward: U128);
    ///
    fn change_maximum_validator_count(&mut self, value: u32);
    ///
    fn change_min_length_of_validator_set_history(&mut self, min_length: U64);
    ///
    fn change_min_interval_for_new_validator_set(&mut self, min_interval: U64);
    ///
    fn change_vsc_packet_timeout_interval(&mut self, interval: U64);
    ///
    fn change_min_validator_staking_amount(&mut self, amount: U128);
}

impl Default for AnchorSettings {
    fn default() -> Self {
        Self {
            chain_revision_number: U64(0),
            era_reward: U128::from(0),
            max_count_of_validators: 60,
            min_length_of_validator_set_history: U64::from(100),
            min_interval_for_new_validator_set: U64::from(3600 * 1_000_000_000),
            vsc_packet_timeout_interval: U64::from(2400 * 1_000_000_000),
            min_validator_staking_amount: U128::from(10_000_000_000_000_000_000_000_000_000),
        }
    }
}

#[near_bindgen]
impl AnchorSettingsManager for AppchainAnchor {
    //
    fn change_chain_revision_number(&mut self, value: U64) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            value.0 != anchor_settings.chain_revision_number.0,
            "The value is not changed."
        );
        anchor_settings.chain_revision_number = value;
        self.anchor_settings.set(&anchor_settings);
    }
    //
    fn change_era_reward(&mut self, era_reward: U128) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            era_reward.0 != anchor_settings.era_reward.0,
            "The value is not changed."
        );
        anchor_settings.era_reward = era_reward;
        self.anchor_settings.set(&anchor_settings);
    }
    //
    fn change_maximum_validator_count(&mut self, value: u32) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        assert!(value > 0, "The value should be greater than 0.");
        assert!(
            value != anchor_settings.max_count_of_validators,
            "The value is not changed."
        );
        anchor_settings.max_count_of_validators = value;
        self.anchor_settings.set(&anchor_settings);
    }
    //
    fn change_min_length_of_validator_set_history(&mut self, min_length: U64) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            min_length.0 != anchor_settings.min_length_of_validator_set_history.0,
            "The value is not changed."
        );
        anchor_settings.min_length_of_validator_set_history = min_length;
        self.anchor_settings.set(&anchor_settings);
    }
    //
    fn change_min_interval_for_new_validator_set(&mut self, min_interval_secs: U64) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        let min_interval = min_interval_secs.0 * 1_000_000_000;
        assert!(
            min_interval != anchor_settings.min_interval_for_new_validator_set.0,
            "The value is not changed."
        );
        anchor_settings.min_interval_for_new_validator_set = U64::from(min_interval);
        self.anchor_settings.set(&anchor_settings);
    }
    //
    fn change_vsc_packet_timeout_interval(&mut self, interval_secs: U64) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        let interval = interval_secs.0 * 1_000_000_000;
        assert!(
            interval != anchor_settings.vsc_packet_timeout_interval.0,
            "The value is not changed."
        );
        anchor_settings.vsc_packet_timeout_interval = U64::from(interval);
        self.anchor_settings.set(&anchor_settings);
    }
    //
    fn change_min_validator_staking_amount(&mut self, amount: U128) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            amount.0 != anchor_settings.min_validator_staking_amount.0,
            "The value is not changed."
        );
        anchor_settings.min_validator_staking_amount = amount;
        self.anchor_settings.set(&anchor_settings);
    }
}
