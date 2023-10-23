use crate::*;
use base64::{DecodeError, Engine};

/// The interfaces for `restaking-base` contract to call.
pub trait RestakingBaseActions {
    /// Set the validator's public key in the appchain.
    ///
    /// The `key` param should be in bs58 format like
    /// `ed25519:8wKmDwNsF1hPzsDW8ASdU9GuwfSpT93ieTyP7767nLS9`.
    fn bond(&mut self, staker_id: AccountId, key: String);
    /// Change the validator's public key in the appchain.
    ///
    /// The `key` param should be in bs58 format like
    /// `ed25519:8wKmDwNsF1hPzsDW8ASdU9GuwfSpT93ieTyP7767nLS9`.
    fn change_key(&mut self, staker_id: AccountId, key: String);
}

#[near_bindgen]
impl RestakingBaseActions for AppchainAnchor {
    //
    fn bond(&mut self, staker_id: AccountId, key: String) {
        self.assert_restaking_base_contract();
        match decode_pubkey(&key) {
            Ok(public_key) => {
                self.validator_id_to_pubkey_map
                    .insert(&staker_id, &public_key);
                self.validator_address_to_id_map
                    .insert(&calculate_address(public_key.as_slice()), &staker_id);
            }
            Err(err) => panic!("Invalid public key: {}", err),
        };
    }
    //
    fn change_key(&mut self, staker_id: AccountId, key: String) {
        self.assert_restaking_base_contract();
        let old_key = self
            .validator_id_to_pubkey_map
            .get(&staker_id)
            .expect("The staker is not bonded yet.");
        match decode_pubkey(&key) {
            Ok(public_key) => {
                self.validator_id_to_pubkey_map
                    .insert(&staker_id, &public_key);
                self.validator_address_to_id_map
                    .remove(&calculate_address(old_key.as_slice()));
                self.validator_address_to_id_map
                    .insert(&calculate_address(public_key.as_slice()), &staker_id);
            }
            Err(err) => panic!("Invalid public key: {:?}", err),
        };
    }
}

fn decode_pubkey(key: &String) -> Result<Vec<u8>, DecodeError> {
    let key = key.trim_start_matches("ed25519:");
    base64::engine::general_purpose::STANDARD.decode(key)
}
