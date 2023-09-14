use crate::*;
use core::str::FromStr;

/// The interfaces for `restaking-base` contract to call.
pub trait RestakingBaseActions {
    /// Set the validator's public key in the appchain.
    ///
    /// The `pubkey` param should be in bs58 format like
    /// `ed25519:8wKmDwNsF1hPzsDW8ASdU9GuwfSpT93ieTyP7767nLS9`.
    fn bond(&mut self, staker_id: AccountId, key: String);
    /// Change the validator's public key in the appchain.
    fn change_key(&mut self, staker_id: AccountId, key: String);
}

#[near_bindgen]
impl RestakingBaseActions for AppchainAnchor {
    //
    #[payable]
    fn bond(&mut self, staker_id: AccountId, key: String) {
        self.assert_restaking_base_contract();
        match PublicKey::from_str(key.as_str()) {
            Ok(public_key) => {
                let storage_cost = (staker_id.as_bytes().len() + public_key.as_bytes().len())
                    as u128
                    * env::storage_byte_cost();
                assert!(
                    env::attached_deposit() >= storage_cost,
                    "Not enough deposit to cover the storage cost. At least needs {} yocto.",
                    storage_cost
                );
                self.validator_pubkeys_in_appchain
                    .insert(&staker_id, &public_key.into_bytes())
            }
            Err(err) => panic!("Invalid public key: {}", err),
        };
    }
    //
    fn change_key(&mut self, staker_id: AccountId, key: String) {
        self.assert_restaking_base_contract();
        assert!(
            self.validator_pubkeys_in_appchain.contains_key(&staker_id),
            "The staker is not bonded yet."
        );
        match PublicKey::from_str(key.as_str()) {
            Ok(public_key) => self
                .validator_pubkeys_in_appchain
                .insert(&staker_id, &public_key.into_bytes()),
            Err(err) => panic!("Invalid public key: {}", err),
        };
    }
}
