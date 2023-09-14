use crate::*;
use near_sdk::borsh::maybestd::collections::HashMap;
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
pub struct AppchainSettings {
    pub rpc_endpoint: String,
    pub subql_endpoint: String,
    pub era_reward: U128,
    pub bonus_for_new_validator: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AnchorSettings {
    pub token_price_maintainer_account: Option<AccountId>,
    pub relayer_account: Option<AccountId>,
    pub beefy_light_client_witness_mode: bool,
    pub min_length_of_validator_set_history: U64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProtocolSettings {
    /// A validator has to deposit a certain amount of OCT token to this contract for
    /// being validator of the appchain.
    pub minimum_validator_deposit: U128,
    /// The minimum amount for a validator to increase or decrease his/her deposit.
    pub minimum_validator_deposit_changing_amount: U128,
    /// The maximum percent value that the deposit of a validator in total stake.
    pub maximum_validator_stake_percent: u16,
    /// The minimum deposit amount for a delegator to delegate his voting weight to
    /// a certain validator.
    pub minimum_delegator_deposit: U128,
    /// The minimum amount for a delegator to increase or decrease his/her delegation
    /// to a validator.
    pub minimum_delegator_deposit_changing_amount: U128,
    /// The minimum price (in USD) of total stake in this contract for
    /// booting corresponding appchain.
    pub minimum_total_stake_price_for_booting: U128,
    /// The maximum percentage of the total market value of all NEP-141 tokens to the total
    /// market value of OCT token staked in this contract.
    pub maximum_market_value_percent_of_near_fungible_tokens: u16,
    /// The maximum percentage of the total market value of wrapped appchain token to the total
    /// market value of OCT token staked in this contract.
    pub maximum_market_value_percent_of_wrapped_appchain_token: u16,
    /// The minimum number of validator(s) registered in this contract for
    /// booting the corresponding appchain and keep it alive.
    pub minimum_validator_count: U64,
    /// The maximum number of validator(s) registered in this contract for
    /// the corresponding appchain.
    pub maximum_validator_count: U64,
    /// The maximum number of validator(s) which a delegator can delegate to.
    pub maximum_validators_per_delegator: U64,
    /// The unlock period (in days) for validator(s) can withdraw their deposit after
    /// they are removed from the corresponding appchain.
    pub unlock_period_of_validator_deposit: U64,
    /// The unlock period (in days) for delegator(s) can withdraw their deposit after
    /// they no longer delegates their stake to a certain validator on the corresponding appchain.
    pub unlock_period_of_delegator_deposit: U64,
    /// The maximum number of historical eras that the validators or delegators are allowed to
    /// withdraw their reward.
    pub maximum_era_count_of_unwithdrawn_reward: U64,
    /// The maximum number of valid appchain message.
    /// If the era number of appchain message is smaller than the latest era number minus
    /// this value, the message will be considered as `invalid`.
    pub maximum_era_count_of_valid_appchain_message: U64,
    /// The percent of commission fees of a validator's reward in an era.
    pub validator_commission_percent: u16,
    /// The maximum unprofitable era count for auto-unbonding a validator.
    pub maximum_allowed_unprofitable_era_count: u16,
    /// The subaccount name for council keeper contract.
    pub subaccount_for_council_keeper_contract: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainValidator {
    pub validator_id: AccountId,
    pub validator_pubkey_in_appchain: PublicKey,
    pub total_stake: U128,
}

/// The actual processing order is:
/// `CopyingFromLastEra` -> `UnbondingValidator`-> `AutoUnbondingValidator`
/// -> `ApplyingStakingHistory` -> `SyncingStakingAmountToCouncil` -> `ReadyForDistributingReward`
/// -> `DistributingReward`
/// -> `CheckingForAutoUnbondingValidator` -> `Completed`
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ValidatorSetProcessingStatus {
    CopyingFromLastEra {
        copying_validator_index: U64,
        copying_delegator_index: U64,
    },
    ApplyingStakingHistory {
        applying_index: U64,
    },
    ReadyForDistributingReward,
    DistributingReward {
        appchain_message_nonce: u32,
        distributing_validator_index: U64,
        distributing_delegator_index: U64,
    },
    Completed,
    UnbondingValidator {
        unbonding_validator_index: U64,
        unbonding_delegator_index: U64,
    },
    AutoUnbondingValidator {
        unbonding_validator_index: U64,
        unbonding_delegator_index: U64,
    },
    CheckingForAutoUnbondingValidator {
        unprofitable_validator_index: U64,
    },
    SyncingStakingAmountToCouncil,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PermissionlessActionsStatus {
    /// The era number that is switching by permissionless actions
    pub switching_era_number: Option<U64>,
    /// The era number that is distributing reward by permissionless actions
    pub distributing_reward_era_number: Option<U64>,
    ///
    pub processing_appchain_message_nonce: Option<u32>,
    ///
    pub max_nonce_of_staged_appchain_messages: u32,
    ///
    pub latest_applied_appchain_message_nonce: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct IndexRange {
    pub start_index: U64,
    pub end_index: U64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardHistory {
    pub era_number: U64,
    pub total_reward: U128,
    pub unwithdrawn_reward: U128,
    pub expired: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct DelegationRewardHistory {
    pub era_number: U64,
    pub delegated_validator: AccountId,
    pub total_reward: U128,
    pub unwithdrawn_reward: U128,
    pub expired: bool,
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
    /// The status of creation of this set
    pub processing_status: ValidatorSetProcessingStatus,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorProfile {
    ///
    pub validator_id: AccountId,
    ///
    pub validator_id_in_appchain: String,
    ///
    pub profile: HashMap<String, String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainNotification {
    /// A certain amount of a NEAR fungible token has been locked in appchain anchor.
    NearFungibleTokenLocked {
        contract_account: String,
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    },
    /// A certain amount of wrapped appchain token is burnt in its contract in NEAR protocol.
    WrappedAppchainTokenBurnt {
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    },
    /// A certain wrapped non-fungible token is burnt in its contract in NEAR protocol.
    WrappedNonFungibleTokenBurnt {
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        class_id: String,
        instance_id: String,
    },
    /// A certain wrapped appchain NFT is locked in appchain anchor.
    WrappedAppchainNFTLocked {
        class_id: String,
        token_id: String,
        sender_id_in_near: AccountId,
        owner_id_in_near: AccountId,
        receiver_id_in_appchain: String,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainNotificationHistory {
    pub appchain_notification: AppchainNotification,
    pub block_height: U64,
    pub timestamp: U64,
    pub index: U64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainMessageProcessingResult {
    Ok { nonce: u32, message: Option<String> },
    Error { nonce: u32, message: String },
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
