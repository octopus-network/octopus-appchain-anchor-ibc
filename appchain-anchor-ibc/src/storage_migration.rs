use crate::{validator_set::Validator, *};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{env, near_bindgen, AccountId, IntoStorageKey, Timestamp};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
#[borsh(crate = "near_sdk::borsh")]
pub struct OldValidatorSet {
    /// The id of the validator set.
    id: u64,
    /// The set of account id of validators.
    validator_id_set: UnorderedSet<AccountId>,
    /// The validators data, mapped by their account id in NEAR protocol.
    validators: LookupMap<AccountId, Validator>,
    /// Total stake of current set
    total_stake: Balance,
    /// The sequence of the validator set in restaking base contract.
    sequence: u64,
    /// The timestamp of when this validator set is created.
    timestamp: Timestamp,
    /// Whether the validator set is matured in the corresponding appchain.
    matured_in_appchain: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, Debug)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct OldRewardDistribution {
    pub validator_set_id: U64,
    pub amount: U128,
    pub timestamp: Timestamp,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[borsh(crate = "near_sdk::borsh")]
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
    /// The minimum staking amount of a quliafied validator.
    pub min_validator_staking_amount: U128,
}

pub trait StorageMigration {
    fn migrate_state() -> Self;
}

#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
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
    pending_rewards: LookupArray<RewardDistribution>,
    /// The pending slash packets received from near-ibc contract.
    pending_slash_packets: LookupArray<String>,
}

#[near_bindgen]
impl StorageMigration for AppchainAnchor {
    #[init(ignore_state)]
    fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OldAppchainAnchor = env::state_read().expect("Old state doesn't exist");
        near_sdk::assert_self();
        //
        // Migrate actions by old contract data.
        //
        let index_range = old_contract.validator_set_histories.index_range();
        for index in index_range.start_index.0..index_range.end_index.0 + 1 {
            let old_vs_data = env::storage_read(&get_storage_key_in_lookup_array(
                &StorageKey::ValidatorSetHistories,
                &index,
            ));
            if let Some(old_vs_data) = old_vs_data {
                let old_vs: OldValidatorSet =
                    near_sdk::borsh::BorshDeserialize::try_from_slice(&old_vs_data).unwrap();
                let new_vs = ValidatorSet::from_old_version(old_vs);
                let new_vs_data = near_sdk::borsh::to_vec(&new_vs).unwrap();
                env::storage_write(
                    &get_storage_key_in_lookup_array(&StorageKey::ValidatorSetHistories, &index),
                    &new_vs_data,
                );
            }
        }
        let index_range = old_contract.pending_rewards.index_range();
        for index in index_range.start_index.0..index_range.end_index.0 + 1 {
            let old_rd_data = env::storage_read(&get_storage_key_in_lookup_array(
                &StorageKey::PendingRewards,
                &index,
            ));
            if let Some(old_rd_data) = old_rd_data {
                let old_rd: OldRewardDistribution =
                    near_sdk::borsh::BorshDeserialize::try_from_slice(&old_rd_data).unwrap();
                let new_rd = RewardDistribution::from_old_version(old_rd);
                let new_rd_data = near_sdk::borsh::to_vec(&new_rd).unwrap();
                env::storage_write(
                    &get_storage_key_in_lookup_array(&StorageKey::PendingRewards, &index),
                    &new_rd_data,
                );
            }
        }
        let old_anchor_settings_data =
            env::storage_read(&StorageKey::AnchorSettings.into_storage_key());
        if let Some(old_anchor_settings_data) = old_anchor_settings_data {
            let old_anchor_settings: OldAnchorSettings =
                near_sdk::borsh::BorshDeserialize::try_from_slice(&old_anchor_settings_data)
                    .unwrap();
            let new_anchor_settings = AnchorSettings {
                chain_revision_number: old_anchor_settings.chain_revision_number,
                era_reward: old_anchor_settings.era_reward,
                max_count_of_validators: old_anchor_settings.max_count_of_validators,
                min_length_of_validator_set_history: old_anchor_settings
                    .min_length_of_validator_set_history,
                min_interval_for_new_validator_set: old_anchor_settings
                    .min_interval_for_new_validator_set,
                vsc_packet_timeout_interval: old_anchor_settings.vsc_packet_timeout_interval,
                min_validator_staking_amount: old_anchor_settings.min_validator_staking_amount,
                min_unjail_interval: U64::from(600 * 1_000_000_000),
                appchain_address_bech32_hrp: "ottovalcons".to_string(),
            };
            let new_anchor_settings_data = near_sdk::borsh::to_vec(&new_anchor_settings).unwrap();
            env::storage_write(
                &StorageKey::AnchorSettings.into_storage_key(),
                &new_anchor_settings_data,
            );
        }
        //
        // Create the new contract using the data from the old contract.
        //
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
            anchor_settings: old_contract.anchor_settings,
            appchain_state: old_contract.appchain_state,
            pending_rewards: old_contract.pending_rewards,
            pending_slash_packets: old_contract.pending_slash_packets,
        };
        //
        // Migrate actions by new contract data.
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
    result.extend(near_sdk::borsh::to_vec(index).unwrap());
    result
}

impl ValidatorSet {
    pub fn from_old_version(old_vs: OldValidatorSet) -> Self {
        ValidatorSet {
            id: old_vs.id,
            validator_id_set: old_vs.validator_id_set,
            validators: old_vs.validators,
            total_stake: old_vs.total_stake,
            sequence: old_vs.sequence,
            timestamp: old_vs.timestamp,
            matured_in_appchain: old_vs.matured_in_appchain,
            jailed_validators: vec![],
        }
    }
}

impl RewardDistribution {
    pub fn from_old_version(old_rd: OldRewardDistribution) -> Self {
        RewardDistribution {
            validator_set_id: old_rd.validator_set_id,
            amount: old_rd.amount,
            timestamp: old_rd.timestamp,
            distributed: false,
        }
    }
}
