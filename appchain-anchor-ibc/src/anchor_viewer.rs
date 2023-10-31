use base64::Engine;

use crate::*;

pub trait AnchorViewer {
    /// Get the chain id of corresponding appchain.
    fn get_chain_id(&self) -> ChainId;
    /// Get version of this contract.
    fn get_anchor_version(&self) -> String;
    /// Get owner of this contract.
    fn get_owner(&self) -> AccountId;
    /// Get anchor settings detail.
    fn get_anchor_settings(&self) -> AnchorSettings;
    /// Get the reward token contract.
    fn get_reward_token_contract(&self) -> AccountId;
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
    fn get_chain_id(&self) -> ChainId {
        ChainId::new(
            self.appchain_id.as_str(),
            self.anchor_settings.get().unwrap().chain_revision_number.0,
        )
        .expect("INVALID_CHAIN_ID, should not happen")
    }
    //
    fn get_anchor_version(&self) -> String {
        ANCHOR_VERSION.to_string()
    }
    //
    fn get_owner(&self) -> AccountId {
        self.owner.clone()
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
            locked_reward_token_amount: self.locked_reward_token_amount.into(),
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
            .map(|vs| self.get_validator_set_view_of(&vs))
    }
}

impl AppchainAnchor {
    //
    fn get_validator_set_view_of(&self, validator_set: &ValidatorSet) -> ValidatorSetView {
        ValidatorSetView {
            id: U64::from(validator_set.id()),
            validators: validator_set
                .get_validator_ids()
                .iter()
                .map(|id| {
                    if let Some(validator) = validator_set.get_validator(&id) {
                        ValidatorView {
                            validator_id: validator.validator_id,
                            total_stake: validator.total_stake,
                            status: validator.status,
                            registered_pubkey: self.validator_id_to_pubkey_map.get(&id).map_or(
                                String::new(),
                                |bytes| {
                                    format!(
                                        "ed25519:{}",
                                        base64::engine::general_purpose::STANDARD.encode(&bytes)
                                    )
                                },
                            ),
                        }
                    } else {
                        unreachable!()
                    }
                })
                .collect(),
            total_stake: validator_set.total_stake(),
            sequence: U64::from(validator_set.sequence()),
            timestamp: validator_set.timestamp(),
            matured_on_appchain: validator_set.matured_in_appchain(),
        }
    }
}
