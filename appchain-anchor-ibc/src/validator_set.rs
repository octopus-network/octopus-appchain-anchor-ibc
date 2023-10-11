use crate::*;
use near_sdk::{IntoStorageKey, Timestamp};

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ValidatorStatus {
    /// A validator with this status will be sent to appchain in the next VSC packet.
    Active,
    /// The validator is jailed by appchain by downtime slashing.
    ///
    /// Will be set when a slash packet with `infraction == 1` is received.
    /// The validator will be included in the next validator set, but keep this status.
    ///
    /// A validator with this status will not be sent to appchain in the next VSC packet.
    ///
    /// A jailed validator can call the `unjail_validator` function to unjail itself (to change
    /// its status to `Active`).
    Jailed,
    /// The validator is waiting for slash by appchain governance.
    ///
    /// Will be set when a slash packet with `infraction == 2` is received.
    /// The validator will be included in the next validator set, but keep this status.
    ///
    /// A validator with this status will not be sent to appchain in the next VSC packet.
    WaitForSlash,
    /// The validator is slashed by appchain governance.
    ///
    /// Will be set when function `approve_slash_request` is called.
    /// The validator will be included in the next validator set, but keep this status.
    ///
    /// A validator with this status will not be sent to appchain in the next VSC packet.
    Slashed,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Deserialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Validator {
    /// The validator's id in NEAR protocol.
    pub validator_id: AccountId,
    /// Total stake of the validator, including delegations of all delegators.
    pub total_stake: Balance,
    /// Whether the validator is slashed.
    pub status: ValidatorStatus,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct ValidatorSet {
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
    ///
    timestamp: Timestamp,
    /// Whether the validator set is matured in the corresponding appchain.
    matured_in_appchain: bool,
}

pub trait ValidatorSetViewer {
    ///
    fn contains_validator(&self, validator_id: &AccountId) -> bool;
    ///
    fn get_validator(&self, validator_id: &AccountId) -> Option<Validator>;
    ///
    fn get_validator_by_index(&self, index: &u64) -> Option<Validator>;
    ///
    fn get_validator_ids(&self) -> Vec<AccountId>;
    ///
    fn id(&self) -> u64;
    ///
    fn sequence(&self) -> u64;
    ///
    fn timestamp(&self) -> Timestamp;
    ///
    fn matured_in_appchain(&self) -> bool;
    ///
    fn total_stake(&self) -> u128;
    ///
    fn validator_count(&self) -> u64;
    ///
    fn active_validators(&self) -> Vec<(AccountId, U128)>;
    ///
    fn slash_ack_validators(&self) -> Vec<AccountId>;
}

impl ValidatorSet {
    ///
    pub fn new(id: u64, sequence: u64) -> Self {
        Self {
            id: 0,
            validator_id_set: UnorderedSet::new(
                StorageKey::ValidatorIdSetOf(id).into_storage_key(),
            ),
            validators: LookupMap::new(StorageKey::ValidatorsOf(id).into_storage_key()),
            total_stake: 0,
            sequence,
            timestamp: env::block_timestamp(),
            matured_in_appchain: false,
        }
    }
    ///
    pub fn add_validator(&mut self, validator_id: AccountId, stake: Balance) {
        if self.validator_id_set.insert(&validator_id) {
            self.validators.insert(
                &validator_id,
                &Validator {
                    validator_id: validator_id.clone(),
                    total_stake: stake,
                    status: ValidatorStatus::Active,
                },
            );
            self.total_stake += stake;
        }
    }
    ///
    pub fn clear(&mut self, max_gas: Gas) -> ProcessingResult {
        let validator_ids = self.validator_id_set.to_vec();
        for validator_id in validator_ids {
            self.validators.remove(&validator_id);
            self.validator_id_set.remove(&validator_id);
            if env::used_gas() > max_gas {
                return ProcessingResult::NeedMoreGas;
            }
        }
        self.total_stake = 0;
        ProcessingResult::Ok
    }
    ///
    pub fn set_matured(&mut self) {
        self.matured_in_appchain = true;
    }
}

impl ValidatorSetViewer for ValidatorSet {
    //
    fn contains_validator(&self, validator_id: &AccountId) -> bool {
        self.validators.contains_key(validator_id)
    }
    //
    fn get_validator(&self, validator_id: &AccountId) -> Option<Validator> {
        self.validators.get(validator_id)
    }
    //
    fn get_validator_by_index(&self, index: &u64) -> Option<Validator> {
        match self.validator_id_set.as_vector().get(*index) {
            Some(validator_id) => self.validators.get(&validator_id),
            None => None,
        }
    }
    //
    fn get_validator_ids(&self) -> Vec<AccountId> {
        self.validator_id_set.to_vec()
    }
    //
    fn id(&self) -> u64 {
        self.id
    }
    //
    fn sequence(&self) -> u64 {
        self.sequence
    }
    //
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    //
    fn matured_in_appchain(&self) -> bool {
        self.matured_in_appchain
    }
    //
    fn total_stake(&self) -> u128 {
        self.total_stake
    }
    //
    fn validator_count(&self) -> u64 {
        self.validator_id_set.len()
    }
    //
    fn active_validators(&self) -> Vec<(AccountId, U128)> {
        self.validator_id_set
            .iter()
            .filter(|id| {
                if let Some(validator) = self.validators.get(&id) {
                    validator.status == ValidatorStatus::Active
                } else {
                    false
                }
            })
            .map(|id| {
                if let Some(validator) = self.validators.get(&id) {
                    (
                        validator.validator_id.clone(),
                        U128::from(validator.total_stake),
                    )
                } else {
                    unreachable!()
                }
            })
            .collect()
    }
    //
    fn slash_ack_validators(&self) -> Vec<AccountId> {
        self.validator_id_set
            .iter()
            .filter(|id| {
                if let Some(validator) = self.validators.get(&id) {
                    validator.status == ValidatorStatus::Jailed
                        || validator.status == ValidatorStatus::WaitForSlash
                } else {
                    false
                }
            })
            .collect()
    }
}

impl IndexedAndClearable for ValidatorSet {
    ///
    fn set_index(&mut self, _index: &u64) {
        ()
    }
    ///
    fn clear_extra_storage(&mut self, max_gas: Gas) -> ProcessingResult {
        self.clear(max_gas)
    }
}

impl From<ValidatorSet> for ValidatorSetView {
    fn from(validator_set: ValidatorSet) -> Self {
        Self {
            id: U64::from(validator_set.id),
            validators: validator_set
                .validator_id_set
                .iter()
                .map(|id| {
                    if let Some(validator) = validator_set.validators.get(&id) {
                        validator.clone()
                    } else {
                        unreachable!()
                    }
                })
                .collect(),
            total_stake: validator_set.total_stake,
            sequence: U64::from(validator_set.sequence),
            timestamp: validator_set.timestamp,
            matured_on_appchain: validator_set.matured_in_appchain,
        }
    }
}
