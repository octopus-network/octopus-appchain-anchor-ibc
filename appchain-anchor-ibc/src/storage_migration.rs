use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{env, near_bindgen, AccountId, Balance, IntoStorageKey};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldAppchainAnchor {
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
    /// The custom settings for appchain.
    appchain_settings: LazyOption<AppchainSettings>,
    /// The anchor settings for appchain.
    anchor_settings: LazyOption<AnchorSettings>,
    /// The protocol settings for appchain anchor.
    protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain.
    appchain_state: AppchainState,
}

#[near_bindgen]
impl AppchainAnchor {
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OldAppchainAnchor = env::state_read().expect("Old state doesn't exist");
        //
        near_sdk::assert_self();
        //
        // Create the new contract using the data from the old contract.
        let new_contract = AppchainAnchor {
            appchain_id: old_contract.appchain_id,
            appchain_registry: old_contract.appchain_registry,
            owner: old_contract.owner,
            restaking_base_contract: old_contract.restaking_base_contract,
            lpos_market_contract: old_contract.lpos_market_contract,
            near_ibc_contract: old_contract.near_ibc_contract,
            reward_token_contract: old_contract.reward_token_contract,
            locked_reward_token_amount: old_contract.locked_reward_token_amount,
            validator_set_histories: old_contract.validator_set_histories,
            validator_pubkeys_in_appchain: old_contract.validator_pubkeys_in_appchain,
            appchain_settings: old_contract.appchain_settings,
            anchor_settings: old_contract.anchor_settings,
            protocol_settings: old_contract.protocol_settings,
            appchain_state: old_contract.appchain_state,
        };
        //
        //
        new_contract
    }
}

pub fn get_storage_key_in_lookup_array<T: BorshSerialize>(
    prefix: &StorageKey,
    index: &T,
) -> Vec<u8> {
    let mut result = prefix.clone().into_storage_key();
    result.extend(index.try_to_vec().unwrap());
    result
}
