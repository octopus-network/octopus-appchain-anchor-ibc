use crate::*;
use near_sdk::{env, json_types::Base58CryptoHash, IntoStorageKey, NearToken};

const GAS_FOR_UPGRADE_SELF_DEPLOY: Gas = Gas::from_tgas(15);

/// Stores attached data into blob store and returns hash of it.
/// Implemented to avoid loading the data into WASM for optimal gas usage.
#[no_mangle]
pub extern "C" fn store_wasm_of_self() {
    env::setup_panic_hook();
    let contract: AppchainAnchor = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
    contract.assert_owner();
    let input = env::input().expect("ERR_NO_INPUT");
    let sha256_hash = env::sha256(&input);

    let blob_len = input.len();
    let storage_cost = ((blob_len + 32) as u128) * env::storage_byte_cost().as_yoctonear();
    assert!(
        env::attached_deposit().as_yoctonear() >= storage_cost,
        "ERR_NOT_ENOUGH_DEPOSIT:{}",
        storage_cost
    );

    env::storage_write(&StorageKey::AnchorContractWasm.into_storage_key(), &input);
    let mut blob_hash = [0u8; 32];
    blob_hash.copy_from_slice(&sha256_hash);
    let blob_hash_str = near_sdk::serde_json::to_string(&Base58CryptoHash::from(blob_hash))
        .unwrap()
        .into_bytes();

    env::value_return(&blob_hash_str);
}

#[no_mangle]
pub fn update_self() {
    env::setup_panic_hook();
    let contract: AppchainAnchor = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
    contract.assert_owner();
    let current_id = env::current_account_id();
    let input = env::storage_read(&StorageKey::AnchorContractWasm.into_storage_key())
        .expect("Wasm file for deployment is not staged yet.");
    let promise_id = env::promise_batch_create(&current_id);
    env::promise_batch_action_deploy_contract(promise_id, &input);
    env::promise_batch_action_function_call(
        promise_id,
        "migrate_state",
        &[],
        NearToken::from_yoctonear(0),
        env::prepaid_gas()
            .checked_sub(env::used_gas())
            .expect("Prepaid gas is too little.")
            .checked_sub(GAS_FOR_UPGRADE_SELF_DEPLOY)
            .expect("Prepaid gas is too little."),
    );
}
