use crate::*;
use near_sdk::IntoStorageKey;

pub type AppchainId = String;

/// The state of an appchain
#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainState {
    /// The initial state of an appchain, after it is successfully registered.
    /// This state is managed by appchain registry.
    Registered,
    /// The state while the appchain is under auditing by Octopus Network.
    /// This state is managed by appchain registry.
    Audited,
    /// The state while members of Octopus DAO can upvote for the appchain.
    /// This state is managed by appchain registry.
    Voting,
    /// The state while an appchain is booting.
    Booting,
    /// The state while an appchain is active normally.
    Active,
    /// The state which an appchain is closing for some technical or governance reasons.
    Closing,
    /// The state which the lifecycle of an appchain is end.
    Closed,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AnchorSettings {
    /// The rewards amount for each era.
    pub era_reward: U128,
    /// The maximum number of validator(s) registered in this contract for
    /// the corresponding appchain.
    pub maximum_validator_count: U64,
    /// The minimum length of validator set history.
    /// This is used for keeping the minimum count of validator set history.
    pub min_length_of_validator_set_history: U64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainValidator {
    pub validator_id: AccountId,
    pub validator_pubkey_in_appchain: PublicKey,
    pub total_stake: U128,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct IndexRange {
    pub start_index: U64,
    pub end_index: U64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AnchorStatus {
    pub total_stake_in_next_era: U128,
    pub validator_count_in_next_era: U64,
    pub index_range_of_validator_set_history: IndexRange,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorSetInfo {
    /// The number of era in appchain.
    pub era_number: U64,
    /// Total stake of current set
    pub total_stake: U128,
    /// The validator list for query
    pub validator_list: Vec<AppchainValidator>,
    /// The block height when the era starts.
    pub start_block_height: U64,
    /// The timestamp when the era starts.
    pub start_timestamp: U64,
    /// The index of the latest staking history happened in the era of corresponding appchain.
    pub staking_history_index: U64,
    /// The set of validator id which will not be profited.
    pub unprofitable_validator_ids: Vec<AccountId>,
    /// Total stake excluding all unprofitable validators' stake.
    pub valid_total_stake: U128,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum MultiTxsOperationProcessingResult {
    NeedMoreGas,
    Ok,
    Error(String),
}

impl MultiTxsOperationProcessingResult {
    ///
    pub fn is_ok(&self) -> bool {
        match self {
            MultiTxsOperationProcessingResult::Ok => true,
            _ => false,
        }
    }
    ///
    pub fn is_need_more_gas(&self) -> bool {
        match self {
            MultiTxsOperationProcessingResult::NeedMoreGas => true,
            _ => false,
        }
    }
    ///
    pub fn is_error(&self) -> bool {
        match self {
            MultiTxsOperationProcessingResult::Error(_) => true,
            _ => false,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub enum RemovingValidatorSetSteps {
    ClearingOldestValidatorSet,
}

impl RemovingValidatorSetSteps {
    ///
    pub fn save(&self) {
        env::storage_write(
            &StorageKey::RemovingValidatorSetSteps.into_storage_key(),
            &self.try_to_vec().unwrap(),
        );
    }
    ///
    pub fn recover() -> Self {
        let bytes = env::storage_read(&StorageKey::RemovingValidatorSetSteps.into_storage_key());
        if let Some(bytes) = bytes {
            Self::try_from_slice(&bytes).unwrap()
        } else {
            Self::ClearingOldestValidatorSet
        }
    }
    ///
    pub fn clear() {
        env::storage_remove(&StorageKey::RemovingValidatorSetSteps.into_storage_key());
    }
}
