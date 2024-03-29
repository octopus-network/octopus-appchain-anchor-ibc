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
use base64::{DecodeError, Engine};
use bech32::ToBase32;
use ibc::core::host::types::identifiers::ChainId;
use lookup_array::{IndexedAndClearable, LookupArray};
use near_contract_standards::fungible_token::Balance;
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet},
    env, ext_contract,
    json_types::{Base64VecU8, U128, U64},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json, AccountId, BorshStorageKey, Gas, NearToken, PanicOnDefault, Promise,
    PromiseOrValue,
};
use serde_json::json;
use types::*;
use validator_set::{ValidatorSet, ValidatorSetViewer};

mod anchor_viewer;
mod contract_actions;
mod ext_contracts;
pub mod lookup_array;
mod permissonless_actions;
mod prelude;
pub mod storage_migration;
pub mod types;
mod upgrade;
mod user_actions;
mod validator_set;

const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Constants for gas.
const T_GAS_FOR_SIMPLE_FUNCTION_CALL: u64 = 10;
const T_GAS_CAP_FOR_MULTI_TXS_PROCESSING: u64 = 130;
/// The scale for converting between `NEAR` and `yoctoNear`.
const NEAR_SCALE: u128 = 1_000_000_000_000_000_000_000_000;

/// Storage keys for collections of sub-struct in main contract
#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey, Clone)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    AnchorContractWasm,
    AnchorSettings,
    PendingRewards,
    RemovingValidatorSetSteps,
    ValidatorAddressToIdMap,
    ValidatorIdToPubkeyMap,
    ValidatorSetHistories,
    ValidatorIdSetOf(u64),
    ValidatorsOf(u64),
    PendingSlashPackets,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
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
    validator_id_to_pubkey_map: UnorderedMap<AccountId, Vec<u8>>,
    /// The addresses of validators in appchain.
    validator_address_to_id_map: UnorderedMap<Vec<u8>, AccountId>,
    /// The anchor settings for appchain.
    anchor_settings: LazyOption<AnchorSettings>,
    /// The state of the corresponding appchain.
    appchain_state: AppchainState,
    /// The pending rewards of validators which are not distributed yet.
    pending_rewards: LookupArray<RewardDistribution>,
    /// The pending slash packets received from near-ibc contract.
    pending_slash_packets: LookupArray<String>,
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
        ChainId::new(appchain_id).expect(
            "Invalid account id for appchain anchor ibc. \
            The subaccount name is not valid in `ibc-rs`.",
        );
        Self {
            appchain_id: appchain_id.to_string(),
            appchain_registry: AccountId::try_from(appchain_registry.to_string()).unwrap(),
            owner: env::current_account_id(),
            restaking_base_contract,
            lpos_market_contract,
            near_ibc_contract,
            reward_token_contract,
            locked_reward_token_amount: 0,
            validator_set_histories: LookupArray::new(StorageKey::ValidatorSetHistories),
            validator_id_to_pubkey_map: UnorderedMap::new(StorageKey::ValidatorIdToPubkeyMap),
            validator_address_to_id_map: UnorderedMap::new(StorageKey::ValidatorAddressToIdMap),
            anchor_settings: LazyOption::new(
                StorageKey::AnchorSettings,
                Some(&AnchorSettings::default()),
            ),
            appchain_state: AppchainState::Booting,
            pending_rewards: LookupArray::new(StorageKey::PendingRewards),
            pending_slash_packets: LookupArray::new(StorageKey::PendingSlashPackets),
        }
    }
    //
    pub fn version(&self) -> String {
        VERSION.to_string()
    }
    //
    pub fn set_owner(&mut self, owner: AccountId) {
        self.assert_owner();
        assert!(!owner.eq(&self.owner), "Owner is not changed.",);
        self.owner = owner;
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
    //
    fn assert_near_ibc_contract(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.near_ibc_contract,
            "This function can only be called by near-ibc contract."
        )
    }
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
        if let Some(latest_validator_set) = self.validator_set_histories.get_last() {
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
                NearToken::from_yoctonear(0),
                Gas::from_tgas(T_GAS_FOR_SIMPLE_FUNCTION_CALL),
            );
        }
    }
}

impl IndexedAndClearable for String {
    //
    fn set_index(&mut self, _index: &u64) {
        ()
    }
    //
    fn clear_extra_storage(&mut self, _max_gas: Gas) -> ProcessingResult {
        ProcessingResult::Ok
    }
}

pub fn decode_ed25519_pubkey(key: &String) -> Result<Vec<u8>, DecodeError> {
    let key = key.trim_start_matches("ed25519:");
    base64::engine::general_purpose::STANDARD.decode(key)
}

/// The input param should be a public key in bytes.
pub fn calculate_address(public_key: &[u8]) -> Vec<u8> {
    let hash = env::sha256(public_key);
    let address = hash.get(..20);
    address.expect("Failed to get address from hash.").to_vec()
}

pub fn calculate_bech32_address(hrp: String, address: Vec<u8>) -> String {
    bech32::encode(hrp.as_str(), address.to_base32(), bech32::Variant::Bech32)
        .expect("Invalid address for calculating bech32 address.")
}

pub fn emit_nep297_event<T: Serialize>(event: &str, data: &T) {
    let result = json!({
        "standard":"nep297",
        "version":"1.0.0",
        "event":event,
        "data":data,
    });
    log!(format!("EVENT_JSON:{}", result.to_string()));
}

#[no_mangle]
pub extern "C" fn remove_storage_keys() {
    env::setup_panic_hook();
    near_sdk::assert_self();
    assert!(
        !env::current_account_id().to_string().ends_with(".near"),
        "This function can not be called on mainnet."
    );

    let input = env::input().unwrap();
    //
    #[derive(Serialize, Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct Args {
        pub keys: Vec<String>,
    }
    //
    let args: Args = serde_json::from_slice(&input).unwrap();
    for key in args.keys {
        let json_str = format!("\"{}\"", key);
        log!(
            "Remove key '{}': {}",
            key,
            env::storage_remove(&serde_json::from_str::<Base64VecU8>(&json_str).unwrap().0)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{test_utils::VMContextBuilder, testing_env};

    #[test]
    fn test_emit_nep297_event() {
        let context = VMContextBuilder::new().build();
        testing_env!(context.clone());
        emit_nep297_event(
            "TEST_EVENT",
            &SlashPacketView {
                validator: None,
                valset_update_id: 1,
                infraction: "infraction".to_string(),
                received_timestamp: 1,
            },
        );
    }
}
