use crate::{ext_contracts::RestakingBaseValidatorSet, *};
use near_sdk::PromiseResult;

#[ext_contract(ext_restaking_base_callbacks)]
pub trait RestakingBaseCallbacks {
    /// Callback function for `get_validator_set` of restaking base contract
    fn get_validator_set_callback(&mut self);
    /// Callback function for `slash_request` of restaking base contract
    fn slash_request_callback(&mut self, slash_items: Vec<(AccountId, U128)>);
}

#[near_bindgen]
impl RestakingBaseCallbacks for AppchainAnchor {
    //
    fn get_validator_set_callback(&mut self) {
        near_sdk::assert_self();
        match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                let restaking_base_vs =
                    near_sdk::serde_json::from_slice::<RestakingBaseValidatorSet>(&value).unwrap();
                self.validator_set_histories
                    .append(&mut self.generate_new_validator_set(restaking_base_vs));
            }
            PromiseResult::Failed => {
                log!("Failed to get validator set from restaking base contract.");
            }
        }
    }
    //
    fn slash_request_callback(&mut self, slash_items: Vec<(AccountId, U128)>) {
        near_sdk::assert_self();
        match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                let slash_id = near_sdk::serde_json::from_slice::<U64>(&value).unwrap();
                let mut latest_vs = self.validator_set_histories.get_last().unwrap();
                latest_vs.wait_for_slashing_validator(&slash_items[0].0, slash_id);
                log!(
                    "Slash request for {:?} is sent to restaking base contract.",
                    slash_items
                );
            }
            PromiseResult::Failed => {
                log!(
                    "Failed to send slash request for {:?} to restaking base contract.",
                    slash_items
                );
            }
        }
    }
}

impl AppchainAnchor {
    fn generate_new_validator_set(
        &self,
        restaking_base_vs: RestakingBaseValidatorSet,
    ) -> ValidatorSet {
        let anchor_settings = self.anchor_settings.get().unwrap();
        let validator_set = ValidatorSet::new(
            &self.validator_set_histories.get_last(),
            &restaking_base_vs,
            anchor_settings.min_validator_staking_amount.0,
        );
        assert!(
            validator_set.active_validators(None).len() > 0,
            "No qualified validator in new validator set with sequence '{}'.",
            validator_set.sequence()
        );
        validator_set
    }
}
