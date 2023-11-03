use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{env, near_bindgen, AccountId, Balance, IntoStorageKey, Timestamp};

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct OldRewardDistribution {
    pub transfer_call_msg: AnchorDepositRewardMsg,
    pub amount: U128,
    pub timestamp: Timestamp,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldAppchainAnchor {
    /// The id of corresponding appchain.
    appchain_id: AppchainId,
    /// The account id of appchain registry contract.
    appchain_registry: AccountId,
    /// The owner account id.
    owner: AccountId,
    /// The account id of staking pool contract.
    restaking_base_contract: AccountId,
    /// The account id of LPOS market contract.
    lpos_market_contract: AccountId,
    /// The account id of near-ibc contract.
    near_ibc_contract: AccountId,
    /// The token contract that will be used to distribute rewards of validators.
    reward_token_contract: AccountId,
    /// The locked reward token amount.
    locked_reward_token_amount: Balance,
    /// The history data of validator set.
    validator_set_histories: LookupArray<ValidatorSet>,
    /// The pubkeys of validators in appchain.
    validator_id_to_pubkey_map: UnorderedMap<AccountId, Vec<u8>>,
    /// The addresses of validators in appchain.
    validator_address_to_id_map: UnorderedMap<Vec<u8>, AccountId>,
    /// The anchor settings for appchain.
    anchor_settings: LazyOption<AnchorSettings>,
    /// The state of the corresponding appchain.
    appchain_state: AppchainState,
    /// The pending rewards of validators which are not distributed yet.
    pending_rewards: LazyOption<VecDeque<OldRewardDistribution>>,
}

#[near_bindgen]
impl AppchainAnchor {
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let mut old_contract: OldAppchainAnchor =
            env::state_read().expect("Old state doesn't exist");
        //
        near_sdk::assert_self();
        //
        let mut pending_rewards = old_contract.pending_rewards.get().unwrap_or_default();
        assert!(old_contract.pending_rewards.remove());
        // Create the new contract using the data from the old contract.
        let mut new_contract = AppchainAnchor {
            appchain_id: old_contract.appchain_id,
            appchain_registry: old_contract.appchain_registry,
            owner: old_contract.owner,
            restaking_base_contract: old_contract.restaking_base_contract,
            lpos_market_contract: old_contract.lpos_market_contract,
            near_ibc_contract: old_contract.near_ibc_contract,
            reward_token_contract: old_contract.reward_token_contract,
            locked_reward_token_amount: old_contract.locked_reward_token_amount,
            validator_set_histories: old_contract.validator_set_histories,
            validator_id_to_pubkey_map: old_contract.validator_id_to_pubkey_map,
            validator_address_to_id_map: old_contract.validator_address_to_id_map,
            anchor_settings: old_contract.anchor_settings,
            appchain_state: old_contract.appchain_state,
            pending_rewards: LookupArray::new(StorageKey::PendingRewards),
        };
        //
        for reward in &mut pending_rewards {
            new_contract
                .pending_rewards
                .append(&mut RewardDistribution {
                    validator_set_id: new_contract
                        .get_validator_set_id_by_sequence(reward.transfer_call_msg.sequence),
                    amount: reward.amount,
                    timestamp: reward.timestamp,
                });
        }
        //
        new_contract
    }
}

pub fn get_storage_key_in_lookup_array<T: BorshSerialize>(
    prefix: &StorageKey,
    index: &T,
) -> Vec<u8> {
    let mut result = prefix.clone().into_storage_key();
    result.extend(index.try_to_vec().unwrap());
    result
}

impl AppchainAnchor {
    fn get_validator_set_id_by_sequence(&self, sequence: U64) -> U64 {
        let index_range = self.validator_set_histories.index_range();
        for index in index_range.start_index.0..index_range.end_index.0 + 1 {
            let validator_set = self.validator_set_histories.get(&index).unwrap();
            if validator_set.sequence() == sequence.0 {
                return index.into();
            }
        }
        panic!("No validator set found for sequence {}", sequence.0);
    }
}
