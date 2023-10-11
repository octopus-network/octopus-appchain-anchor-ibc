use crate::*;

pub trait AnchorViewer {
    /// Get version of this contract.
    fn get_anchor_version(&self) -> String;
    /// Get anchor settings detail.
    fn get_anchor_settings(&self) -> AnchorSettings;
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
    /// Get pending rewards of validators which are not distributed yet.
    fn get_pending_rewards(&self) -> Vec<RewardDistribution>;
    /// Get validator set history by index.
    fn get_validator_set(&self, index: U64) -> Option<ValidatorSetView>;
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
        let latest_vs = self
            .validator_set_histories
            .get_latest()
            .expect("No validator set history found.");
        AnchorStatus {
            total_stake: latest_vs.total_stake().into(),
            validator_count: latest_vs.validator_count().into(),
            index_range_of_validator_set_history: self.validator_set_histories.index_range(),
            matured_in_appchain: latest_vs.matured_in_appchain(),
        }
    }
    //
    fn get_storage_balance(&self) -> U128 {
        U128::from(u128::from(env::storage_usage()) * env::storage_byte_cost())
    }
    //
    fn get_pending_rewards(&self) -> Vec<RewardDistribution> {
        self.pending_rewards
            .get()
            .unwrap_or_default()
            .iter()
            .cloned()
            .collect()
    }
    //
    fn get_validator_set(&self, index: U64) -> Option<ValidatorSetView> {
        self.validator_set_histories
            .get(&index.0)
            .map(|vs| vs.into())
    }
}
