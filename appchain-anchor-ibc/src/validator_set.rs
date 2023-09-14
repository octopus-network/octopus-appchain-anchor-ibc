use crate::*;
use near_sdk::IntoStorageKey;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Validator {
    /// The validator's id in NEAR protocol.
    pub validator_id: AccountId,
    /// The validator's public key in the appchain.
    pub validator_pubkey_in_appchain: PublicKey,
    /// Total stake of the validator, including delegations of all delegators.
    pub total_stake: Balance,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct ValidatorSet {
    /// The id of the validator set.
    set_id: u64,
    /// The set of account id of validators.
    validator_id_set: UnorderedSet<AccountId>,
    /// The validators data, mapped by their account id in NEAR protocol.
    validators: LookupMap<AccountId, Validator>,
    /// Total stake of current set
    total_stake: Balance,
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
    fn set_id(&self) -> u64;
    ///
    fn total_stake(&self) -> u128;
    ///
    fn validator_count(&self) -> u64;
}

impl ValidatorSet {
    ///
    pub fn new(set_id: u64) -> Self {
        Self {
            set_id,
            validator_id_set: UnorderedSet::new(
                StorageKey::ValidatorIdsOfEra(set_id).into_storage_key(),
            ),
            validators: LookupMap::new(StorageKey::ValidatorsOfEra(set_id).into_storage_key()),
            total_stake: 0,
        }
    }
    ///
    pub fn clear(&mut self, max_gas: Gas) -> MultiTxsOperationProcessingResult {
        let validator_ids = self.validator_id_set.to_vec();
        for validator_id in validator_ids {
            self.validators.remove(&validator_id);
            self.validator_id_set.remove(&validator_id);
            if env::used_gas() > max_gas {
                return MultiTxsOperationProcessingResult::NeedMoreGas;
            }
        }
        self.total_stake = 0;
        MultiTxsOperationProcessingResult::Ok
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
    fn set_id(&self) -> u64 {
        self.set_id
    }
    //
    fn total_stake(&self) -> u128 {
        self.total_stake
    }
    //
    fn validator_count(&self) -> u64 {
        self.validator_id_set.len()
    }
}

impl AppchainValidator {
    ///
    pub fn from_validator(validator: Validator) -> Self {
        Self {
            validator_id: validator.validator_id,
            validator_pubkey_in_appchain: validator.validator_pubkey_in_appchain,
            total_stake: U128::from(validator.total_stake),
        }
    }
}

impl IndexedAndClearable for ValidatorSet {
    ///
    fn set_index(&mut self, _index: &u64) {
        ()
    }
    ///
    fn clear_extra_storage(&mut self, max_gas: Gas) -> MultiTxsOperationProcessingResult {
        self.clear(max_gas)
    }
}
