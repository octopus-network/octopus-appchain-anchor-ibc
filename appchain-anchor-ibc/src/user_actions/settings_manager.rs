use crate::*;
use core::convert::From;

pub trait AnchorSettingsManager {
    ///
    fn set_era_reward(&mut self, era_reward: U128);
    ///
    fn change_maximum_validator_count(&mut self, value: u32);
    ///
    fn set_min_length_of_validator_set_history(&mut self, min_length: U64);
    ///
    fn set_min_interval_for_new_validator_set(&mut self, min_interval: U64);
}

impl Default for AnchorSettings {
    fn default() -> Self {
        Self {
            era_reward: U128::from(0),
            max_count_of_validators: 60,
            min_length_of_validator_set_history: U64::from(100),
            min_interval_for_new_validator_set: U64::from(3600 * 1_000_000_000),
        }
    }
}

#[near_bindgen]
impl AnchorSettingsManager for AppchainAnchor {
    //
    fn set_era_reward(&mut self, era_reward: U128) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
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
    fn set_min_length_of_validator_set_history(&mut self, min_length: U64) {
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
    fn set_min_interval_for_new_validator_set(&mut self, min_interval: U64) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            min_interval.0 != anchor_settings.min_interval_for_new_validator_set.0,
            "The value is not changed."
        );
        anchor_settings.min_interval_for_new_validator_set = min_interval;
        self.anchor_settings.set(&anchor_settings);
    }
}
