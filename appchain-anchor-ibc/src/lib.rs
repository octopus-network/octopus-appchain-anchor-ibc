#![no_std]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

use crate::prelude::*;
use core::ops::Mul;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise,
    PromiseOrValue, PublicKey,
};

use lookup_array::{IndexedAndClearable, LookupArray};
use storage_key::StorageKey;
use types::*;
use validator_set::{ValidatorSet, ValidatorSetViewer};

mod anchor_viewer;
mod ext_contracts;
pub mod lookup_array;
mod prelude;
mod storage_key;
pub mod storage_migration;
pub mod types;
mod upgrade;
mod user_actions;
mod validator_set;

/// Version of this contract (the same as in Cargo.toml)
const ANCHOR_VERSION: &str = "v1.0.0";
/// Constants for gas.
const T_GAS_FOR_SYNC_STATE_TO_REGISTRY: u64 = 10;
const T_GAS_CAP_FOR_MULTI_TXS_PROCESSING: u64 = 130;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AppchainAnchor {
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
    validator_pubkeys_in_appchain: LookupMap<AccountId, Vec<u8>>,
    /// The anchor settings for appchain.
    anchor_settings: LazyOption<AnchorSettings>,
    /// The state of the corresponding appchain.
    appchain_state: AppchainState,
}

#[near_bindgen]
impl AppchainAnchor {
    #[init]
    pub fn new(
        restaking_base_contract: AccountId,
        lpos_market_contract: AccountId,
        near_ibc_contract: AccountId,
        reward_token_contract: AccountId,
    ) -> Self {
        let account_id = env::current_account_id().to_string();
        let parts = account_id.split(".").collect::<Vec<&str>>();
        assert!(
            parts.len() > 2,
            "This contract must be deployed as a sub-account of octopus appchain registry.",
        );
        let (appchain_id, appchain_registry) = account_id.split_once(".").unwrap();
        Self {
            appchain_id: appchain_id.to_string(),
            appchain_registry: AccountId::try_from(appchain_registry.to_string()).unwrap(),
            owner: env::predecessor_account_id(),
            restaking_base_contract,
            lpos_market_contract,
            near_ibc_contract,
            reward_token_contract,
            locked_reward_token_amount: 0,
            validator_set_histories: LookupArray::new(StorageKey::ValidatorSetHistoriesMap),
            validator_pubkeys_in_appchain: LookupMap::new(StorageKey::ValidatorPubkeysInAppchain),
            anchor_settings: LazyOption::new(
                StorageKey::AnchorSettings,
                Some(&AnchorSettings::default()),
            ),
            appchain_state: AppchainState::Booting,
        }
    }
    // Assert that the function is called by the owner.
    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "This function can only be called by owner."
        );
    }
    //
    fn assert_reward_token_contract(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.reward_token_contract,
            "This function can only be called by reward token contract."
        )
    }
    //
    fn assert_restaking_base_contract(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.restaking_base_contract,
            "This function can only be called by restaking base contract."
        )
    }
}

#[near_bindgen]
impl AppchainAnchor {
    //
    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }
    //
    pub fn set_owner(&mut self, owner: AccountId) {
        self.assert_owner();
        assert!(!owner.eq(&self.owner), "Owner is not changed.",);
        self.owner = owner;
    }
}

#[near_bindgen]
impl AppchainAnchor {
    /// Callback function for `ft_transfer_call` of NEP-141 compatible contracts
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.assert_reward_token_contract();
        log!(
            "Deposit {} from '@{}' received. msg: '{}'",
            amount.0,
            &sender_id,
            msg
        );
        self.locked_reward_token_amount += amount.0;
        PromiseOrValue::Value(U128(0))
    }
}

impl AppchainAnchor {
    ///
    pub fn sync_state_to_registry(&self) {
        if let Some(latest_validator_set) = self.validator_set_histories.get_latest() {
            // sync state to appchain registry contract
            #[derive(near_sdk::serde::Serialize)]
            #[serde(crate = "near_sdk::serde")]
            struct Args {
                appchain_id: AppchainId,
                appchain_state: AppchainState,
                validator_count: u32,
                total_stake: U128,
            }
            let args = Args {
                appchain_id: self.appchain_id.clone(),
                appchain_state: self.appchain_state.clone(),
                validator_count: latest_validator_set.validator_count().try_into().unwrap(),
                total_stake: U128::from(latest_validator_set.total_stake()),
            };
            let args = near_sdk::serde_json::to_vec(&args)
                .expect("Failed to serialize the cross contract args using JSON.");
            Promise::new(self.appchain_registry.clone()).function_call(
                "sync_state_of".to_string(),
                args,
                0,
                Gas::ONE_TERA.mul(T_GAS_FOR_SYNC_STATE_TO_REGISTRY),
            );
        }
    }
}
