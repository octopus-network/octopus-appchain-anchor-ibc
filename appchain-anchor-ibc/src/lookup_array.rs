use crate::*;
use core::fmt::Debug;

pub trait IndexedAndClearable {
    ///
    fn set_index(&mut self, index: &u64);
    ///
    fn clear_extra_storage(&mut self, max_gas: Gas) -> ProcessingResult;
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct LookupArray<T: BorshDeserialize + BorshSerialize + Debug + IndexedAndClearable> {
    /// The anchor event data map.
    pub lookup_map: LookupMap<u64, T>,
    /// The start index of valid anchor event.
    pub start_index: u64,
    /// The end index of valid anchor event.
    pub end_index: u64,
}

/// View functions
impl<T> LookupArray<T>
where
    T: BorshDeserialize + BorshSerialize + Debug + IndexedAndClearable,
{
    ///
    pub fn len(&self) -> u64 {
        if self.end_index > self.start_index {
            return self.end_index - self.start_index + 1;
        } else if self.contains(&self.start_index) {
            return 1;
        } else {
            return 0;
        }
    }
    ///
    pub fn get(&self, index: &u64) -> Option<T> {
        self.lookup_map.get(index)
    }
    ///
    pub fn get_first(&self) -> Option<T> {
        self.lookup_map.get(&self.start_index)
    }
    ///
    pub fn get_latest(&self) -> Option<T> {
        self.lookup_map.get(&self.end_index)
    }
    ///
    pub fn get_slice_of(&self, start_index: &u64, quantity: Option<u64>) -> Vec<T> {
        let mut results = Vec::<T>::new();
        let start_index = match self.start_index > *start_index {
            true => self.start_index,
            false => *start_index,
        };
        let mut end_index = start_index
            + match quantity {
                Some(quantity) => match quantity > 50 {
                    true => 49,
                    false => quantity - 1,
                },
                None => 49,
            };
        end_index = match end_index < self.end_index {
            true => end_index,
            false => self.end_index,
        };
        for index in start_index..end_index + 1 {
            if let Some(record) = self.get(&index) {
                results.push(record);
            }
        }
        results
    }
    ///
    pub fn contains(&self, index: &u64) -> bool {
        self.lookup_map.contains_key(index)
    }
    ///
    pub fn index_range(&self) -> IndexRange {
        IndexRange {
            start_index: U64::from(self.start_index),
            end_index: U64::from(self.end_index),
        }
    }
    ///
    pub fn to_vec(&self) -> Vec<T> {
        let mut results = Vec::<T>::new();
        for index in self.start_index..self.end_index + 1 {
            if let Some(record) = self.get(&index) {
                results.push(record);
            }
        }
        results
    }
}

/// Change functions
impl<T> LookupArray<T>
where
    T: BorshDeserialize + BorshSerialize + Debug + IndexedAndClearable,
{
    ///
    pub fn new(storage_key: StorageKey) -> Self {
        Self {
            lookup_map: LookupMap::new(storage_key),
            start_index: 0,
            end_index: 0,
        }
    }
    ///
    pub fn migrate_from(storage_key: StorageKey, start_index: u64, end_index: u64) -> Self {
        Self {
            lookup_map: LookupMap::new(storage_key),
            start_index,
            end_index,
        }
    }
    ///
    pub fn update(&mut self, index: &u64, record: &T) {
        assert!(
            index >= &self.start_index && index <= &self.end_index,
            "Invalid index for updating in lookup array: {}",
            index
        );
        self.lookup_map.insert(index, record);
    }
    ///
    pub fn append(&mut self, record: &mut T) -> T {
        let index = if self.start_index == 0 && !self.lookup_map.contains_key(&0) {
            0
        } else {
            self.end_index + 1
        };
        record.set_index(&index);
        self.lookup_map.insert(&index, record);
        self.end_index = index;
        self.lookup_map.get(&index).unwrap()
    }
    ///
    pub fn remove_before(&mut self, index: &u64, max_gas: Gas) -> ProcessingResult {
        if self.start_index >= *index {
            return ProcessingResult::Ok;
        }
        for index in self.start_index..*index {
            let result = self.remove_at(&index, max_gas);
            if !result.is_ok() {
                return result;
            }
        }
        self.start_index = *index;
        ProcessingResult::Ok
    }
    ///
    pub fn reset_to(&mut self, index: &u64, max_gas: Gas) -> ProcessingResult {
        assert!(
            *index >= self.start_index && *index <= self.end_index,
            "Invalid history data index."
        );
        for index in (*index + 1)..self.end_index + 1 {
            let result = self.remove_at(&index, max_gas);
            if !result.is_ok() {
                return result;
            }
        }
        self.end_index = *index;
        ProcessingResult::Ok
    }
    ///
    pub fn clear(&mut self, max_gas: Gas) -> ProcessingResult {
        log!(
            "Index range of lookup array: {} - {}",
            self.start_index,
            self.end_index
        );
        let mut index = self.start_index;
        while index <= self.end_index && self.remove_at(&index, max_gas).is_ok() {
            index += 1;
        }
        if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
            self.start_index = index;
            ProcessingResult::NeedMoreGas
        } else {
            self.start_index = 0;
            self.end_index = 0;
            ProcessingResult::Ok
        }
    }
    ///
    pub fn remove_at(&mut self, index: &u64, max_gas: Gas) -> ProcessingResult {
        let result = match self.lookup_map.get(index) {
            Some(mut record) => {
                let result = record.clear_extra_storage(max_gas);
                match result {
                    ProcessingResult::Ok => {
                        self.lookup_map.remove_raw(&index.try_to_vec().unwrap());
                    }
                    ProcessingResult::NeedMoreGas => {
                        self.lookup_map.insert(index, &record);
                    }
                    ProcessingResult::Error(_) => (),
                }
                result
            }
            None => ProcessingResult::Ok,
        };
        if result.is_ok() {
            if *index == self.start_index && *index < self.end_index {
                self.start_index += 1;
            } else if *index == self.end_index && *index > self.start_index {
                self.end_index -= 1;
            } else if *index == self.start_index && *index == self.end_index {
                self.start_index = 0;
                self.end_index = 0;
            }
        }
        result
    }
    ///
    pub fn remove_first(&mut self, max_gas: Gas) -> ProcessingResult {
        self.remove_at(&self.start_index.clone(), max_gas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{test_utils::VMContextBuilder, testing_env};

    #[test]
    fn test_lookup_array() {
        let context = VMContextBuilder::new().build();
        testing_env!(context.clone());
        let max_gas = Gas::ONE_TERA.mul(10);
        let mut lookup_array = LookupArray::<RewardDistribution>::new(StorageKey::PendingRewards);
        // case 1
        lookup_array.append(&mut RewardDistribution {
            validator_set_id: U64::from(0),
            amount: U128::from(100),
            timestamp: env::block_timestamp(),
        });
        lookup_array.append(&mut RewardDistribution {
            validator_set_id: U64::from(1),
            amount: U128::from(101),
            timestamp: env::block_timestamp(),
        });
        assert_eq!(lookup_array.len(), 2);
        assert_eq!(lookup_array.index_range().start_index, U64::from(0));
        assert_eq!(lookup_array.index_range().end_index, U64::from(1));
        assert_eq!(
            lookup_array.get_first().unwrap().validator_set_id,
            U64::from(0)
        );
        lookup_array.remove_first(max_gas);
        assert_eq!(lookup_array.len(), 1);
        assert_eq!(lookup_array.index_range().start_index, U64::from(1));
        assert_eq!(lookup_array.index_range().end_index, U64::from(1));
        lookup_array.remove_first(max_gas);
        assert_eq!(lookup_array.len(), 0);
        assert_eq!(lookup_array.index_range().start_index, U64::from(0));
        assert_eq!(lookup_array.index_range().end_index, U64::from(0));
        // case 2
        lookup_array.append(&mut RewardDistribution {
            validator_set_id: U64::from(0),
            amount: U128::from(100),
            timestamp: env::block_timestamp(),
        });
        lookup_array.append(&mut RewardDistribution {
            validator_set_id: U64::from(1),
            amount: U128::from(101),
            timestamp: env::block_timestamp(),
        });
        lookup_array.append(&mut RewardDistribution {
            validator_set_id: U64::from(2),
            amount: U128::from(102),
            timestamp: env::block_timestamp(),
        });
        assert_eq!(lookup_array.len(), 3);
        assert_eq!(lookup_array.index_range().start_index, U64::from(0));
        assert_eq!(lookup_array.index_range().end_index, U64::from(2));
        assert_eq!(
            lookup_array.get_first().unwrap().validator_set_id,
            U64::from(0)
        );
        assert_eq!(
            lookup_array.get_latest().unwrap().validator_set_id,
            U64::from(2)
        );
        lookup_array.clear(max_gas);
        assert_eq!(lookup_array.len(), 0);
        assert_eq!(lookup_array.index_range().start_index, U64::from(0));
        assert_eq!(lookup_array.index_range().end_index, U64::from(0));
        lookup_array.remove_first(max_gas);
        assert_eq!(lookup_array.len(), 0);
        assert_eq!(lookup_array.index_range().start_index, U64::from(0));
        assert_eq!(lookup_array.index_range().end_index, U64::from(0));
    }
}
