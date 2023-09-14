use near_contract_standards::{
    fungible_token::metadata::FungibleTokenMetadata,
    non_fungible_token::metadata::NFTContractMetadata,
};

use crate::*;

pub trait ProtocolSettingsManager {
    ///
    fn change_minimum_validator_deposit(&mut self, value: U128);
    ///
    fn change_minimum_validator_deposit_changing_amount(&mut self, value: U128);
    ///
    fn change_maximum_validator_stake_percent(&mut self, value: u16);
    ///
    fn change_minimum_delegator_deposit(&mut self, value: U128);
    ///
    fn change_minimum_delegator_deposit_changing_amount(&mut self, value: U128);
    ///
    fn change_minimum_total_stake_price_for_booting(&mut self, value: U128);
    ///
    fn change_maximum_market_value_percent_of_near_fungible_tokens(&mut self, value: u16);
    ///
    fn change_maximum_market_value_percent_of_wrapped_appchain_token(&mut self, value: u16);
    ///
    fn change_minimum_validator_count(&mut self, value: U64);
    ///
    fn change_maximum_validator_count(&mut self, value: U64);
    ///
    fn change_maximum_validators_per_delegator(&mut self, value: U64);
    ///
    fn change_unlock_period_of_validator_deposit(&mut self, value: U64);
    ///
    fn change_unlock_period_of_delegator_deposit(&mut self, value: U64);
    ///
    fn change_maximum_era_count_of_unwithdrawn_reward(&mut self, value: U64);
    ///
    fn change_maximum_era_count_of_valid_appchain_message(&mut self, value: U64);
    ///
    fn change_validator_commission_percent(&mut self, value: u16);
    ///
    fn change_maximum_allowed_unprofitable_era_count(&mut self, value: u16);
    ///
    fn change_subaccount_for_council_keeper_contract(&mut self, subaccount_name: String);
}

pub trait AppchainSettingsManager {
    ///
    fn set_rpc_endpoint(&mut self, rpc_endpoint: String);
    ///
    fn set_subql_endpoint(&mut self, subql_endpoint: String);
    ///
    fn set_era_reward(&mut self, era_reward: U128);
    ///
    fn set_bonus_for_new_validator(&mut self, bonus_amount: U128);
}

pub trait AnchorSettingsManager {
    ///
    fn set_token_price_maintainer_account(&mut self, account_id: AccountId);
    ///
    fn set_relayer_account(&mut self, account_id: AccountId);
    ///
    fn turn_on_beefy_light_client_witness_mode(&mut self);
    ///
    fn turn_off_beefy_light_client_witness_mode(&mut self);
    ///
    fn set_min_length_of_validator_set_history(&mut self, min_length: U64);
}

pub trait WrappedAppchainTokenManager {
    ///
    fn sync_basedata_of_wrapped_appchain_token(
        &mut self,
        metadata: FungibleTokenMetadata,
        premined_beneficiary: AccountId,
        premined_balance: U128,
    );
    ///
    fn set_account_of_wrapped_appchain_token(&mut self, contract_account: AccountId);
    ///
    fn set_total_supply_of_wrapped_appchain_token(&mut self, total_supply: U128);
    ///
    fn set_price_of_wrapped_appchain_token(&mut self, price: U128);
    ///
    fn burn_wrapped_appchain_token(&self, receiver_id: String, amount: U128);
}

pub trait WrappedAppchainNFTManager {
    ///
    fn register_wrapped_appchain_nft(&mut self, class_id: String, metadata: NFTContractMetadata);
    ///
    fn change_wrapped_appchain_nft_contract_metadata(
        &mut self,
        class_id: String,
        metadata: NFTContractMetadata,
    );
    ///
    fn open_bridging_of_wrapped_appchain_nft(&mut self, class_id: String);
    ///
    fn close_bridging_of_wrapped_appchain_nft(&mut self, class_id: String);
}

pub trait NativeNearTokenManager {
    ///
    fn deploy_near_vault_contract(&mut self);
    ///
    fn set_price_of_native_near_token(&mut self, price: U128);
    ///
    fn open_bridging_of_native_near_token(&mut self);
    ///
    fn close_bridging_of_native_near_token(&mut self);
    ///
    fn generate_appchain_notification_for_near_deposit(
        &mut self,
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    );
}
