use crate::*;
use core::str::FromStr;

pub trait ValidatorActions {
    /// Set the validator's public key in the appchain.
    ///
    /// The `pubkey` param should be in bs58 format like
    /// `ed25519:8wKmDwNsF1hPzsDW8ASdU9GuwfSpT93ieTyP7767nLS9`.
    ///
    /// Any account can call this function to set their pubkey in appchain
    /// for further use, but they need to deposit enough a certain balance
    /// for the storage cost.
    fn set_validator_pubkey_in_appchain(&mut self, pubkey: String);
}

#[near_bindgen]
impl ValidatorActions for AppchainAnchor {
    //
    #[payable]
    fn set_validator_pubkey_in_appchain(&mut self, pubkey: String) {
        let validator_id = env::predecessor_account_id();
        match PublicKey::from_str(pubkey.as_str()) {
            Ok(public_key) => {
                let storage_cost = (validator_id.as_bytes().len() + public_key.as_bytes().len())
                    as u128
                    * env::storage_byte_cost();
                assert!(
                    env::attached_deposit() >= storage_cost,
                    "Not enough deposit to cover the storage cost. At least needs {} yocto.",
                    storage_cost
                );
                self.validator_pubkeys_in_appchain
                    .insert(&validator_id, &public_key.into_bytes())
            }
            Err(err) => panic!("Invalid public key: {}", err),
        };
    }
}
