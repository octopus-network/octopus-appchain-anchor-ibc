use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RestakingBaseValidatorSet {
    pub validator_set: Vec<(AccountId, U128)>,
    pub sequence: U64,
}

#[ext_contract(ext_restaking_base)]
pub trait RestakingBaseContract {
    /// View function for querying the latest validator set of a certain consumer chain.
    ///
    /// Should return up to `limit` number of validators.
    ///
    /// The returned validators should be in descending order of their staking amounts.
    fn get_validator_set(&self, consumer_chain_id: String, limit: u32)
        -> RestakingBaseValidatorSet;
    /// Request to slash a certain amount of tokens from a certain validator.
    fn slash_request(
        &mut self,
        consumer_chain_id: String,
        slash_items: Vec<(AccountId, U128)>,
        evidence_sha256_hash: String,
    ) -> U64;
}

#[ext_contract(ext_near_ibc)]
pub trait NearIbcContract {
    /// Start sending vsc packet from `near-ibc` contract to the appchain
    /// corresponding to this contract.
    fn send_vsc_packet(&mut self, vsc_packet_data: VscPacketData);
}
