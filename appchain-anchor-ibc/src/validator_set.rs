use crate::{ext_contracts::RestakingBaseValidatorSet, *};
use near_sdk::{IntoStorageKey, Timestamp};

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, Deserialize, Serialize, Clone)]
#[borsh(crate = "near_sdk::borsh")]
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
    ///
    /// The slash request will be sent to restaking base contract by `slash_validator` function.
    /// And the slash id returned by the function will be set to this status.
    WaitForSlash(U64),
    /// The validator is slashed by appchain governance.
    ///
    /// Will be set when function `approve_slash_request` is called.
    /// The validator will be included in the next validator set, but keep this status.
    ///
    /// A validator with this status will not be sent to appchain in the next VSC packet.
    Slashed,
    /// The validator is not qualified to be a validator.
    ///
    /// Normally, the staking amount of the validator is less than the minimum staking amount.
    Unqualified,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Deserialize, Serialize, Clone)]
#[borsh(crate = "near_sdk::borsh")]
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
#[borsh(crate = "near_sdk::borsh")]
pub struct ValidatorSet {
    /// The id of the validator set.
    pub id: u64,
    /// The set of account id of validators.
    pub validator_id_set: UnorderedSet<AccountId>,
    /// The validators data, mapped by their account id in NEAR protocol.
    pub validators: LookupMap<AccountId, Validator>,
    /// Total stake of current set
    pub total_stake: Balance,
    /// The sequence of the validator set in restaking base contract.
    pub sequence: u64,
    /// The timestamp of when this validator set is created.
    pub timestamp: Timestamp,
    /// Whether the validator set is matured in the corresponding appchain.
    pub matured_in_appchain: bool,
    /// The jailed validators with their account id, jailed time and unjailed time.
    pub jailed_validators: Vec<(AccountId, Timestamp, Timestamp)>,
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
    fn jailed_validators(&self) -> Vec<(AccountId, Timestamp, Timestamp)>;
    ///
    fn slash_ack_validators(&self) -> Vec<AccountId>;
}

impl ValidatorSet {
    ///
    pub fn new(
        last_vs: &Option<ValidatorSet>,
        restaking_base_vs: &RestakingBaseValidatorSet,
        min_validator_staking_amount: u128,
    ) -> Self {
        if let Some(last_vs) = last_vs {
            let id = last_vs.id + 1;
            let mut jailed_validators = Vec::new();
            for (id, jt, ut) in last_vs.jailed_validators.iter() {
                if *ut == 0 {
                    jailed_validators.push((id.clone(), *jt, *ut));
                }
            }
            let mut new_instance = Self {
                id,
                validator_id_set: UnorderedSet::new(
                    StorageKey::ValidatorIdSetOf(id).into_storage_key(),
                ),
                validators: LookupMap::new(StorageKey::ValidatorsOf(id).into_storage_key()),
                total_stake: 0,
                sequence: restaking_base_vs.sequence.0,
                timestamp: env::block_timestamp(),
                matured_in_appchain: false,
                jailed_validators,
            };
            for (validator_id, stake) in &restaking_base_vs.validator_set {
                new_instance.add_validator(
                    validator_id.clone(),
                    stake.0,
                    if stake.0 >= min_validator_staking_amount {
                        if let Some(validator) = last_vs.get_validator(&validator_id) {
                            validator.status.clone()
                        } else {
                            ValidatorStatus::Active
                        }
                    } else {
                        ValidatorStatus::Unqualified
                    },
                );
            }
            new_instance
        } else {
            Self {
                id: 0,
                validator_id_set: UnorderedSet::new(
                    StorageKey::ValidatorIdSetOf(0).into_storage_key(),
                ),
                validators: LookupMap::new(StorageKey::ValidatorsOf(0).into_storage_key()),
                total_stake: 0,
                sequence: restaking_base_vs.sequence.0,
                timestamp: env::block_timestamp(),
                matured_in_appchain: false,
                jailed_validators: Vec::new(),
            }
        }
    }
    ///
    pub fn add_validator(
        &mut self,
        validator_id: AccountId,
        stake: Balance,
        status: ValidatorStatus,
    ) {
        if self.validator_id_set.insert(&validator_id) {
            self.validators.insert(
                &validator_id,
                &Validator {
                    validator_id: validator_id.clone(),
                    total_stake: stake,
                    status,
                },
            );
            self.total_stake += stake;
        } else {
            panic!(
                "Validator already exists in validator set {}: {}",
                self.id, validator_id
            );
        }
    }
    ///
    pub fn jail_validator(&mut self, validator_id: &AccountId) {
        if let Some(validator) = self.validators.get(validator_id) {
            if validator.status == ValidatorStatus::Active {
                self.validators.insert(
                    &validator_id,
                    &Validator {
                        validator_id: validator_id.clone(),
                        total_stake: validator.total_stake,
                        status: ValidatorStatus::Jailed,
                    },
                );
                self.update_jailed_timestamp(validator_id);
            } else {
                panic!("Validator is not active: {}", validator_id)
            }
        } else {
            panic!("Validator not found: {}", validator_id)
        }
    }
    ///
    pub fn unjail_validator(&mut self, validator_id: &AccountId, min_unjail_interval: u64) {
        if let Some(validator) = self.validators.get(validator_id) {
            if validator.status == ValidatorStatus::Jailed {
                self.validators.insert(
                    &validator_id,
                    &Validator {
                        validator_id: validator_id.clone(),
                        total_stake: validator.total_stake,
                        status: ValidatorStatus::Active,
                    },
                );
                self.update_unjailed_timestamp(validator_id, min_unjail_interval);
            } else {
                panic!("Validator is not jailed: {}", validator_id)
            }
        } else {
            panic!("Validator not found: {}", validator_id)
        }
    }
    ///
    pub fn wait_for_slashing_validator(&mut self, validator_id: &AccountId, slash_id: U64) {
        if let Some(validator) = self.validators.get(validator_id) {
            if validator.status == ValidatorStatus::Active {
                self.validators.insert(
                    &validator_id,
                    &Validator {
                        validator_id: validator_id.clone(),
                        total_stake: validator.total_stake,
                        status: ValidatorStatus::WaitForSlash(slash_id),
                    },
                );
            }
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
    fn jailed_validators(&self) -> Vec<(AccountId, Timestamp, Timestamp)> {
        self.jailed_validators.clone()
    }
    //
    fn slash_ack_validators(&self) -> Vec<AccountId> {
        self.validator_id_set
            .iter()
            .filter(|id| {
                if let Some(validator) = self.validators.get(&id) {
                    match validator.status {
                        ValidatorStatus::Jailed | ValidatorStatus::WaitForSlash(_) => true,
                        _ => false,
                    }
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

impl ValidatorSet {
    //
    fn update_jailed_timestamp(&mut self, validator_id: &AccountId) {
        for (id, _, _) in self.jailed_validators.iter() {
            if id == validator_id {
                panic!("Validator already jailed: {}", validator_id);
            }
        }
        self.jailed_validators
            .push((validator_id.clone(), env::block_timestamp(), 0));
    }
    //
    fn update_unjailed_timestamp(&mut self, validator_id: &AccountId, min_unjail_interval: u64) {
        for (id, jt, ut) in self.jailed_validators.iter_mut() {
            if id == validator_id {
                if *jt + min_unjail_interval * 1_000_000_000 > env::block_timestamp() {
                    panic!("Validator is not jailed for long enough: {}", validator_id);
                } else {
                    *ut = env::block_timestamp();
                }
                return;
            }
        }
        panic!("Validator not found: {}", validator_id);
    }
}
