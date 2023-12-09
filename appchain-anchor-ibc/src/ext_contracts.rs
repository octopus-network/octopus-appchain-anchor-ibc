use crate::*;
use ibc::core::client::types::Height;
use tendermint::{time::Time as TmTime, Hash};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RestakingBaseValidatorSet {
    pub validator_set: Vec<(AccountId, U128)>,
    pub sequence: U64,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub struct TmConsensusState {
    pub timestamp: TmTime,
    pub root: Vec<u8>,
    pub next_validators_hash: Hash,
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
    /// Create tendermint client in `near-ibc` contract for the appchain corresponding to
    /// this contract.
    fn create_tendermint_client_for_appchain(
        &mut self,
        chain_id: ChainId,
        initial_height: Height,
        trusting_period: U64,
        unbonding_period: U64,
        max_clock_drift: U64,
        upgrade_path: Vec<String>,
        consensus_state: TmConsensusState,
    );
    /// Start sending vsc packet from `near-ibc` contract to the appchain
    /// corresponding to this contract.
    fn send_vsc_packet(
        &mut self,
        chain_id: ChainId,
        vsc_packet_data: VscPacketData,
        timeout_timestamp_interval: U64,
    );
}
