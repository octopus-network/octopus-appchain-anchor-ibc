use crate::{validator_set::ValidatorStatus, *};
use near_sdk::{IntoStorageKey, Timestamp};
use octopus_lpos::packet::consumer::Validator;

pub type AppchainId = String;

/// The state of an appchain
#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, PartialEq)]
#[borsh(crate = "near_sdk::borsh")]
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
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct AnchorSettings {
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
    /// The ninimum time interval for the jailed validators can be unjailed (in nanoseconds).
    pub min_unjail_interval: U64,
    /// The HRP of bech32 address in corresponding appchain.
    pub appchain_address_bech32_hrp: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainValidator {
    pub validator_id: AccountId,
    pub validator_address: Vec<u8>,
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
    pub total_stake: U128,
    pub validator_count: U64,
    pub index_range_of_validator_set_history: IndexRange,
    pub locked_reward_token_amount: U128,
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
pub enum ProcessingResult {
    NeedMoreGas,
    Ok,
    Error(String),
}

impl ProcessingResult {
    ///
    pub fn is_ok(&self) -> bool {
        match self {
            ProcessingResult::Ok => true,
            _ => false,
        }
    }
    ///
    pub fn is_need_more_gas(&self) -> bool {
        match self {
            ProcessingResult::NeedMoreGas => true,
            _ => false,
        }
    }
    ///
    pub fn is_error(&self) -> bool {
        match self {
            ProcessingResult::Error(_) => true,
            _ => false,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
#[borsh(crate = "near_sdk::borsh")]
pub enum RemovingValidatorSetSteps {
    ClearingOldestValidatorSet,
}

impl RemovingValidatorSetSteps {
    ///
    pub fn save(&self) {
        env::storage_write(
            &StorageKey::RemovingValidatorSetSteps.into_storage_key(),
            &near_sdk::borsh::to_vec(self).unwrap(),
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

#[derive(Deserialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorView {
    /// The validator's id in NEAR protocol.
    pub validator_id: AccountId,
    /// Total stake of the validator, including delegations of all delegators.
    pub total_stake: U128,
    /// The voting power of the validator.
    pub voting_power: U64,
    /// Whether the validator is slashed.
    pub status: ValidatorStatus,
    /// The public key the validator registered in anchor contract.
    pub registered_pubkey: String,
    /// The address of the validator in appchain.
    pub address_in_appchain: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorSetView {
    /// The id of the validator set.
    pub id: U64,
    /// All validators in this validator set.
    pub validators: Vec<ValidatorView>,
    /// Total stake of current set
    pub total_stake: U128,
    /// The sequence of the validator set in restaking base contract.
    pub sequence: U64,
    ///
    pub timestamp: Timestamp,
    /// Whether the validator set is matured on appchain.
    pub matured_on_appchain: bool,
    /// The jailed validators with their account id, jailed time and unjailed time.
    pub jailed_validators: Vec<(AccountId, Timestamp, Timestamp)>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum FtTransferMessage {
    AnchorDepositRewardMsg(AnchorDepositRewardMsg),
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, Debug)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct AnchorDepositRewardMsg {
    pub consumer_chain_id: String,
    pub validator_set: Vec<(AccountId, U128)>,
    pub sequence: U64,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize, Debug)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct RewardDistribution {
    pub validator_set_id: U64,
    pub amount: U128,
    pub timestamp: Timestamp,
    pub distributed: bool,
}

impl IndexedAndClearable for RewardDistribution {
    //
    fn set_index(&mut self, _index: &u64) {
        ()
    }
    //
    fn clear_extra_storage(&mut self, _max_gas: Gas) -> ProcessingResult {
        ProcessingResult::Ok
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorKeyAndPower {
    pub public_key: Vec<u8>,
    pub power: U64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct VscPacketData {
    pub validator_pubkeys: Vec<ValidatorKeyAndPower>,
    pub validator_set_id: U64,
    pub slash_acks: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SlashPacketView {
    pub validator: Option<Validator>,
    /// map to the infraction block height on the provider
    pub valset_update_id: u64,
    /// tell if the slashing is for a downtime or a double-signing infraction
    pub infraction: String,
    pub received_timestamp: Timestamp,
}
