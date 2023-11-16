use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{env, near_bindgen, AccountId, Balance, IntoStorageKey};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OldAnchorSettings {
    /// The revision number of corresponding appchain.
    pub chain_revision_number: U64,
    /// The rewards amount for each era.
    pub era_reward: U128,
    /// The maximum number of validator(s) registered in this contract for
    /// the corresponding appchain.
    pub max_count_of_validators: u32,
    /// The minimum length of validator set history.
    /// This is used for keeping the minimum count of validator set history.
    pub min_length_of_validator_set_history: U64,
    /// The minimum interval for new validator set.
    pub min_interval_for_new_validator_set: U64,
    /// The timeout interval for vsc packet (in nanoseconds).
    pub vsc_packet_timeout_interval: U64,
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
    anchor_settings: LazyOption<OldAnchorSettings>,
    /// The state of the corresponding appchain.
    appchain_state: AppchainState,
    /// The pending rewards of validators which are not distributed yet.
    pending_rewards: LookupArray<RewardDistribution>,
}

#[near_bindgen]
impl AppchainAnchor {
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OldAppchainAnchor = env::state_read().expect("Old state doesn't exist");
        //
        near_sdk::assert_self();
        //
        let old_anchor_settings = old_contract.anchor_settings.get().unwrap();
        // Create the new contract using the data from the old contract.
        let new_contract = AppchainAnchor {
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
            anchor_settings: LazyOption::new(
                StorageKey::AnchorSettings,
                Some(&AnchorSettings::from(old_anchor_settings)),
            ),
            appchain_state: old_contract.appchain_state,
            pending_rewards: old_contract.pending_rewards,
        };
        //
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

impl From<OldAnchorSettings> for AnchorSettings {
    fn from(old: OldAnchorSettings) -> Self {
        Self {
            chain_revision_number: old.chain_revision_number,
            era_reward: old.era_reward,
            max_count_of_validators: old.max_count_of_validators,
            min_length_of_validator_set_history: old.min_length_of_validator_set_history,
            min_interval_for_new_validator_set: old.min_interval_for_new_validator_set,
            vsc_packet_timeout_interval: old.vsc_packet_timeout_interval,
            min_validator_staking_amount: U128::from(10_000_000_000_000_000_000_000_000_000),
        }
    }
}
