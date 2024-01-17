use crate::{
    anchor_viewer::AnchorViewer,
    ext_contracts::{ext_near_ibc, TmConsensusState},
    permissonless_actions::PermissionlessActions,
    *,
};
use ibc::core::client::types::Height;
use prost::Message;
use tendermint::time::Time as TmTime;
use tendermint_proto::{abci::ValidatorUpdate, crypto::PublicKey};

const NANO_SECONDS_PER_SECOND: u64 = 1_000_000_000;

pub trait AppchainLifecycleManager {
    /// Create client in `near-ibc` contract for the appchain corresponding to this contract.
    fn create_client_for_appchain(
        &mut self,
        initial_height: Height,
        trusting_period: U64,
        unbonding_period: U64,
        max_clock_drift: U64,
        upgrade_path: Vec<String>,
    );
    /// Verify and change the state of corresponding appchain to `active`.
    fn go_live(&mut self);
}

#[near_bindgen]
impl AppchainLifecycleManager for AppchainAnchor {
    //
    fn create_client_for_appchain(
        &mut self,
        initial_height: Height,
        trusting_period: U64,
        unbonding_period: U64,
        max_clock_drift: U64,
        upgrade_path: Vec<String>,
    ) {
        self.assert_owner();
        assert_eq!(
            self.appchain_state,
            AppchainState::Booting,
            "Appchain state must be 'booting'."
        );
        assert!(
            self.validator_set_histories.get(&0).is_some(),
            "The validator set 0 has not been generated."
        );
        assert!(
            trusting_period.0 < unbonding_period.0,
            "Trusting period must be less than unbonding period."
        );
        assert!(
            max_clock_drift.0 > 0 && max_clock_drift.0 < trusting_period.0,
            "Max clock drift must be greater than 0 and less than trusting period."
        );
        let init_vs = self.validator_set_histories.get(&0).unwrap();
        let validators_bytes: Vec<Vec<u8>> = init_vs
            .active_validators(None)
            .iter()
            .map(|(validator_id, stake)| {
                ValidatorUpdate {
                    pub_key: Some(PublicKey {
                        sum: Some(tendermint_proto::crypto::public_key::Sum::Ed25519(
                            self.validator_id_to_pubkey_map.get(validator_id).unwrap(),
                        )),
                    }),
                    power: (stake.0 / NEAR_SCALE) as i64,
                }
                .encode_to_vec()
            })
            .collect();
        log!("Validators bytes: {:?}", validators_bytes);
        let consensus_state = TmConsensusState {
            timestamp: TmTime::from_unix_timestamp(
                (env::block_timestamp() / NANO_SECONDS_PER_SECOND) as i64,
                (env::block_timestamp() % NANO_SECONDS_PER_SECOND) as u32,
            )
            .expect("INVALID_TIMESTAMP, should not happen"),
            root: b"sentinel_root".to_vec(),
            next_validators_hash: tendermint::Hash::from_bytes(
                tendermint::hash::Algorithm::Sha256,
                merkle_hash(&validators_bytes).as_slice(),
            )
            .expect("INVALID_HASH, should not happen"),
        };
        log!(
            "Consensus state to create: {:?}",
            near_sdk::serde_json::to_string(&consensus_state).unwrap()
        );
        ext_near_ibc::ext(self.near_ibc_contract.clone()).create_tendermint_client_for_appchain(
            self.get_chain_id(),
            initial_height,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            upgrade_path,
            consensus_state,
        );
    }
    //
    fn go_live(&mut self) {
        self.assert_owner();
        assert_eq!(
            self.appchain_state,
            AppchainState::Booting,
            "Appchain state must be 'booting'."
        );
        assert!(
            self.validator_set_histories.get(&0).is_some(),
            "The validator set 0 has not been generated."
        );
        let anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            !(anchor_settings.era_reward.0 == 0),
            "Missing appchain settings."
        );
        self.appchain_state = AppchainState::Active;
        self.sync_state_to_registry();
        self.send_vsc_packet_to_appchain();
    }
}

fn merkle_hash(bytes_array: &Vec<Vec<u8>>) -> Vec<u8> {
    match bytes_array.len() {
        0 => empty_hash(),
        1 => leaf_hash(bytes_array[0].as_slice()),
        _ => {
            let k = bytes_array.len().next_power_of_two() / 2;
            let left = merkle_hash(&bytes_array[0..k].to_vec());
            let right = merkle_hash(&bytes_array[k..].to_vec());
            inner_hash(left.as_slice(), right.as_slice())
        }
    }
}

fn empty_hash() -> Vec<u8> {
    env::sha256(Vec::new().as_slice())
}

fn leaf_hash(bytes: &[u8]) -> Vec<u8> {
    let mut leaf_bytes = Vec::<u8>::new();
    leaf_bytes.push(0);
    leaf_bytes.extend_from_slice(bytes);
    env::sha256(leaf_bytes.as_slice())
}

fn inner_hash(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut inner_bytes = Vec::<u8>::new();
    inner_bytes.push(1);
    inner_bytes.extend_from_slice(left);
    inner_bytes.extend_from_slice(right);
    env::sha256(inner_bytes.as_slice())
}
