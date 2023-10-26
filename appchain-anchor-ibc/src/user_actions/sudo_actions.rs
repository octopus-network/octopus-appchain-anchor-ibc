use crate::*;
use near_sdk::IntoStorageKey;

pub trait SudoActions {
    ///
    fn change_appchain_registry(&mut self, appchain_registry: AccountId);
    ///
    fn change_near_ibc_contract(&mut self, nearibc_contract: AccountId);
    ///
    fn set_reward_token_contract(&mut self, account_id: AccountId);
    ///
    fn remove_oldest_validator_set(&mut self) -> String;
    ///
    fn remove_staged_wasm(&mut self);
}

#[near_bindgen]
impl SudoActions for AppchainAnchor {
    //
    fn change_appchain_registry(&mut self, appchain_registry: AccountId) {
        self.assert_owner();
        assert!(
            !self.appchain_registry.eq(&appchain_registry),
            "Appchain registry is not changed.",
        );
        self.appchain_registry = appchain_registry;
    }
    //
    fn change_near_ibc_contract(&mut self, near_ibc_contract: AccountId) {
        self.assert_owner();
        assert!(
            !self.near_ibc_contract.eq(&near_ibc_contract),
            "NearIBC contract is not changed.",
        );
        self.near_ibc_contract = near_ibc_contract;
    }
    //
    fn set_reward_token_contract(&mut self, account_id: AccountId) {
        self.assert_owner();
        assert!(
            !self.reward_token_contract.eq(&account_id),
            "Reward token contract is not changed.",
        );
        self.reward_token_contract = account_id;
    }
    //
    fn remove_oldest_validator_set(&mut self) -> String {
        self.assert_owner();
        let anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            self.validator_set_histories.len()
                > anchor_settings.min_length_of_validator_set_history.0,
            "The length of validator set histories must not be less than {}.",
            anchor_settings.min_length_of_validator_set_history.0
        );
        let max_gas = Gas::ONE_TERA.mul(170);
        let mut era_number = self.validator_set_histories.index_range().start_index;
        while env::used_gas() < max_gas && self.validator_set_histories.get(&era_number.0).is_none()
        {
            self.validator_set_histories.remove_first(max_gas);
            era_number = self.validator_set_histories.index_range().start_index;
        }
        if self.validator_set_histories.len()
            <= anchor_settings.min_length_of_validator_set_history.0
        {
            return format!("Era {}: {:?}", era_number.0, ProcessingResult::Ok);
        }
        let mut result = (ProcessingResult::NeedMoreGas, None);
        while env::used_gas() < max_gas && result.0.is_need_more_gas() {
            result = match RemovingValidatorSetSteps::recover() {
                RemovingValidatorSetSteps::ClearingOldestValidatorSet => {
                    let result = self.validator_set_histories.remove_first(max_gas);
                    if result.is_ok() {
                        RemovingValidatorSetSteps::clear();
                        (result, None)
                    } else {
                        (result, Some(RemovingValidatorSetSteps::recover()))
                    }
                }
            };
        }
        format!("Era {}: {:?}", era_number.0, result)
    }
    //
    fn remove_staged_wasm(&mut self) {
        self.assert_owner();
        log!(
            "AnchorContractWasm: {}",
            env::storage_remove(&StorageKey::AnchorContractWasm.into_storage_key())
        );
    }
}
