use crate::*;

/// The interfaces for `restaking-base` contract to call.
pub trait RestakingBaseActions {
    /// Set the validator's public key in the appchain.
    ///
    /// The `key` param should be in base64 format like
    /// `ed25519:GMaw7UPsXqPr7IRijvt/BgVU93A6hs98JZbUJtKMAuA=`.
    fn bond(&mut self, staker_id: AccountId, key: String);
    /// Change the validator's public key in the appchain.
    ///
    /// The `key` param should be in base64 format like
    /// `ed25519:GMaw7UPsXqPr7IRijvt/BgVU93A6hs98JZbUJtKMAuA=`.
    fn change_key(&mut self, staker_id: AccountId, key: String);
}

#[near_bindgen]
impl RestakingBaseActions for AppchainAnchor {
    //
    fn bond(&mut self, staker_id: AccountId, key: String) {
        self.assert_restaking_base_contract();
        match decode_ed25519_pubkey(&key) {
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
        match decode_ed25519_pubkey(&key) {
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
