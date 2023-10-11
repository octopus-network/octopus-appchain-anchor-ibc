use crate::*;
use core::str::FromStr;

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
                self.validator_id_to_pubkey_map
                    .insert(&staker_id, &public_key.clone().into_bytes());
                self.validator_address_to_id_map
                    .insert(&calculate_address(public_key.as_bytes()), &staker_id);
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
        match PublicKey::from_str(key.as_str()) {
            Ok(public_key) => {
                self.validator_id_to_pubkey_map
                    .insert(&staker_id, &public_key.clone().into_bytes());
                self.validator_address_to_id_map
                    .remove(&calculate_address(old_key.as_slice()));
                self.validator_address_to_id_map
                    .insert(&calculate_address(public_key.as_bytes()), &staker_id);
            }
            Err(err) => panic!("Invalid public key: {}", err),
        };
    }
}
