use crate::*;

pub trait AnchorViewer {
    /// Get version of this contract.
    fn get_anchor_version(&self) -> String;
    /// Get anchor settings detail.
    fn get_anchor_settings(&self) -> AnchorSettings;
    /// Get appchain settings detail.
    fn get_appchain_settings(&self) -> AppchainSettings;
    /// Get protocol settings detail.
    fn get_protocol_settings(&self) -> ProtocolSettings;
    /// Get the reward token contract.
    fn get_reward_token_contract(&self) -> AccountId;
    /// Get the locked amount of reward token.
    fn get_locked_reward_token_amount(&self) -> U128;
    /// Get state of corresponding appchain.
    fn get_appchain_state(&self) -> AppchainState;
    /// Get current status of anchor.
    fn get_anchor_status(&self) -> AnchorStatus;
    /// Get current storage balance needed by this contract account.
    fn get_storage_balance(&self) -> U128;
}

#[near_bindgen]
impl AnchorViewer for AppchainAnchor {
    //
    fn get_anchor_version(&self) -> String {
        ANCHOR_VERSION.to_string()
    }
    //
    fn get_anchor_settings(&self) -> AnchorSettings {
        self.anchor_settings.get().unwrap()
    }
    //
    fn get_appchain_settings(&self) -> AppchainSettings {
        self.appchain_settings.get().unwrap()
    }
    //
    fn get_protocol_settings(&self) -> ProtocolSettings {
        self.protocol_settings.get().unwrap()
    }
    //
    fn get_reward_token_contract(&self) -> AccountId {
        self.reward_token_contract.clone()
    }
    //
    fn get_locked_reward_token_amount(&self) -> U128 {
        U128::from(self.locked_reward_token_amount)
    }
    //
    fn get_appchain_state(&self) -> AppchainState {
        self.appchain_state.clone()
    }
    //
    fn get_anchor_status(&self) -> AnchorStatus {
        let next_validator_set = self.validator_set_histories.get_latest().unwrap();
        AnchorStatus {
            total_stake_in_next_era: next_validator_set.total_stake().into(),
            validator_count_in_next_era: next_validator_set.validator_count().into(),
            index_range_of_validator_set_history: self.validator_set_histories.index_range(),
        }
    }
    //
    fn get_storage_balance(&self) -> U128 {
        U128::from(u128::from(env::storage_usage()) * env::storage_byte_cost())
    }
}
