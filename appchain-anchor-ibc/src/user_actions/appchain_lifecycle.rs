use crate::*;

pub trait AppchainLifecycleManager {
    /// Verify and change the state of corresponding appchain to `active`.
    fn go_live(&mut self);
}

#[near_bindgen]
impl AppchainLifecycleManager for AppchainAnchor {
    //
    fn go_live(&mut self) {
        self.assert_owner();
        assert_eq!(
            self.appchain_state,
            AppchainState::Booting,
            "Appchain state must be 'booting'."
        );
        assert!(
            self.validator_set_histories.get(&0).is_some(),
            "The validator set 0 has not been generated."
        );
        let appchain_settings = self.appchain_settings.get().unwrap();
        assert!(
            !(appchain_settings.rpc_endpoint.trim().is_empty()
                || appchain_settings.era_reward.0 == 0),
            "Missing appchain settings."
        );
        self.appchain_state = AppchainState::Active;
        self.sync_state_to_registry();
    }
}
