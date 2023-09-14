use crate::*;
use core::convert::From;

pub trait AnchorSettingsManager {
    ///
    fn set_era_reward(&mut self, era_reward: U128);
    ///
    fn change_maximum_validator_count(&mut self, value: U64);
    ///
    fn set_min_length_of_validator_set_history(&mut self, min_length: U64);
}

impl Default for AnchorSettings {
    fn default() -> Self {
        Self {
            era_reward: U128::from(0),
            maximum_validator_count: U64::from(60),
            min_length_of_validator_set_history: U64::from(100),
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
    fn change_maximum_validator_count(&mut self, value: U64) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        assert!(value.0 > 0, "The value should be greater than 0.");
        assert!(
            value.0 != anchor_settings.maximum_validator_count.0,
            "The value is not changed."
        );
        anchor_settings.maximum_validator_count = value;
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
}
