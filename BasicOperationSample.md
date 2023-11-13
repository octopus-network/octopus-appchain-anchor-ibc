# Basic Operation Sample of Octopus Appchain Anchor IBC

## Presequisites

* [Restaking Base Contract](https://github.com/octopus-network/restaking-base) is deployed.
* [LPOS Market Contract](https://github.com/octopus-network/lpos_market) is deployed.
* [NEAR IBC Contract](https://github.com/octopus-network/near-ibc) with [Octopus LPOS module](https://github.com/octopus-network/octopus-lpos-ibc) enabled is deployed.
* Reward Token Contract (NEP-141) for corresponding appchain is deployed.

## Setup and Initialization

We assume the accounts of necessary contracts are as follows:

* Octopus Appchain Registry Contract: `octopus-registry.testnet`
* Restaking Base Contract: `restaking-base.testnet`
* LPOS Market Contract: `lpos-market.testnet`
* NEAR IBC Contract: `near-ibc.testnet`
* Reward Token Contract: `reward-token.testnet`

### Register appchain in Octopus Appchain Registry Contract

Register appchain in Octopus Appchain Registry Contract with UI or cli command. Here is an example for cli command:

```bash
near call oct.beta_oct_relay.testnet ft_transfer_call '{"receiver_id":"registry.test_oct.testnet","amount":"1000000000000000000000","memo":null,"msg":"{\"RegisterAppchain\":{\"appchain_id\":\"oct-cosmos-1\",\"description\":\"octopus cosmos chain 1 description\",\"appchain_type\":\"Cosmos\",\"evm_chain_id\":null,\"website_url\":\"http://ddfs.dsdfs\",\"github_address\":\"https://jldfs.yoasdfasd\",\"contact_email\":\"joe@lksdf.com\",\"premined_wrapped_appchain_token_beneficiary\":\"riversyang.testnet\",\"premined_wrapped_appchain_token\":\"1000000000000000000000\",\"initial_supply_of_wrapped_appchain_token\":\"1000000000000000000000\",\"ido_amount_of_wrapped_appchain_token\":\"300000000000000000000\",\"initial_era_reward\":\"2000000000000000\",\"fungible_token_metadata\":{\"spec\":\"ft-1.0.0\",\"name\":\"joeToken\",\"symbol\":\"JOT\",\"icon\":null,\"reference\":null,\"reference_hash\":null,\"decimals\":18},\"custom_metadata\":{\"key1\":\"value1\"}}}"}' --accountId riversyang.testnet --depositYocto 1 --gas 200000000000000
```

> Note: The `appchain id` used in registry contract, anchor contract and restaking base contract should NOT contain the revision number. For example, the `appchain id` in registry contract should be `oct-cosmos-1` instead of `oct-cosmos-1-0`. The revision number is managed in anchor contract and can be changed by `change_chain_revision_number` function. When interacting with near ibc contract, the revision number will be added automatically.

Registry admin needs to pass the auditing for the appchain:

```bash
near call $REGISTRY_ACCOUNT_ID pass_auditing_appchain '{"appchain_id":"oct-cosmos-1"}' --accountId test_oct.testnet
```

Then create a proposal in AstroDAO for the appchain and register the URL with the following admin command:

```bash
near call $REGISTRY_ACCOUNT_ID start_voting_appchain '{"appchain_id":"oct-cosmos-1","dao_proposal_url":"https://testnet.app.astrodao.com/dao/octopus-dao.sputnikv2.testnet/proposals/octopus-dao.sputnikv2.testnet-64"}' --accountId test_oct.testnet
```

When the voting has passed in DAO, the appchain anchor account for the appchain will be created automatically.

### Prepare reward token contract

Deploy a new NEP-141 compatible token contract or use an existing one.

Register appchain anchor account id and Octopus LPOS market contract id to the token contract. Here is an example for cli command:

```bash
near call $REWARD_TOKEN storage_deposit '{"account_id":"oct-cosmos-1.registry.test_oct.testnet","registration_only":null}' --accountId my-account.testnet --deposit 0.00125
near call $REWARD_TOKEN storage_deposit '{"account_id":"contract-2.lpos-market.testnet","registration_only":null}' --accountId my-account.testnet --deposit 0.00125
```

### Deploy appchain anchor contract

Deploy the appchain anchor contract to the account which is created when the registration process in registry contract is completed. Here is an example for cli command:

```bash
near deploy --accountId $ANCHOR_ACCOUNT_ID --initFunction 'new' --initArgs '{"restaking_base_contract":"contract-2.restaking-base.testnet","lpos_market_contract":"contract-2.lpos-market.testnet","near_ibc_contract":"v6.nearibc.testnet","reward_token_contract":"oct.beta_oct_relay.testnet"}' --wasmFile res/appchain_anchor_ibc.wasm
```

### Register appchain in Restaking Base Contract

Register consumer chain in Restaking Base contract with the consumer chain governance account. Here is an example for cli command:

```bash
near call $RESTAKING_BASE register_consumer_chain '{"register_param":{"consumer_chain_id":"cosmos:oct-cosmos-1","cc_pos_account":"oct-cosmos-1.registry.test_oct.testnet","unbond_period":86400,"website":"https://jldfs.yoasdfasd","treasury":"riversyang.testnet"}}' --accountId riversyang.testnet --deposit 0.1 --gas 200000000000000
```

Then the validators in the restaking base contract can restake to the consumer chain by bonding their pubkeys in consumer chain in Octopus LPOS market contract. The key bonded here should be the same as the key used in the validator node for the consumer chain. For example:

```bash
near call $LPOS_MARKET bond '{"consumer_chain_id":"cosmos:oct-cosmos-1","key":"ed25519:lriei60AvqKy1VPOTQzm2Ka8MMxZEEwvONJtZrtBCSU="}' --accountId riversyang.testnet --deposit 0.01 --gas 200000000000000
```

> Note: The key bonded here must be in base64 format and must start with `ed25519:`.

We also need to sync the consumer chain info from restaking base contract to lpos market contract. This command can be called by any account:

```bash
near call $LPOS_MARKET sync_consumer_chain_pos '{"consumer_chain_id":"cosmos:oct-cosmos-1"}' --accountId my-account.testnet --gas 200000000000000
```

### Fetch initial validator set from restaking base contract

There is a permissionless function in anchor contract to fetch the validator set from restaking base contract. Here is an example for cli command:

```bash
near call $ANCHOR_ACCOUNT_ID fetch_validator_set_from_restaking_base '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
```

Then check the latest validator set and validator set 0 by the following view functions:

```bash
near view $ANCHOR_ACCOUNT_ID get_anchor_status ''
near view $ANCHOR_ACCOUNT_ID get_validator_set '{"index":"0"}'
```

### Create client for appchain in NEAR IBC Contract

Use sudo function to create the client for corresponding appchain in NEAR IBC Contract. Here is an example for cli command:

```bash
near call $ANCHOR_ACCOUNT_ID create_client_for_appchain '{"initial_height":{"revision_number":0,"revision_height":1},"trusting_period":"1209600","unbonding_period":"1814400","max_clock_drift":"600","upgrade_path":[]}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
```

> The unbonding period must be the same as the value in genesis file of corresponding appchain. The trusting period normally should be 2/3 of the unbonding period.

### Prepare Octopus VP and hermes

Refer to [Octopus Verification Proxy](https://github.com/octopus-network/verification-proxies) for the details.

Make use you are using the hermes on branch `v1.7.0-octopus-lpos` of [Octopus modified hermes](https://github.com/octopus-network/hermes).

> In `config.toml` of hermes, the chain id should be the same as the chain id in the corresponding client state stored in the near ibc contract. In this example, it should be `oct-cosmos-1-0`.

After setting up the Octopus VP environment, we need to initialize the vp_near for further use.

```bash
hermes create client --host-chain oct-cosmos-1-0 --reference-chain near-0 --only-init-vp true > hermes_init_vp.log 2>&1 | tail -f hermes_init_vp.log
```

Find the public key and timestamp of solomachine client state from the log of the above command, which is necesary for us to modify the genesis file of appchain. The log message may like this:

```text
2023-10-20T01:38:01.261214Z  WARN ThreadId(36) pubkey: "{\"type\":\"tendermint/PubKeySecp256k1\",\"value\":\"Aw85rIfFOh/iQwy5+abdXuIKaccW7tmOjHMXDFCeWkuz\"}"
2023-10-20T01:38:01.261231Z  WARN ThreadId(36) timestamp: 1697741792311761696
```

### Modify the `genesis.json` for the appchain

Modify the `chain_id` of `genesis.json` to the chain id of the appchain and modify the `genesis_time` to proper time. For example:

```json
    "genesis_time": "2023-10-18T20:29:50.804910403Z",
    "chain_id": "oct-cosmos-1-0",
```

Use the solomachine client state info we got in the previous step to modify the `genesis.json` of appchain. Remember to change the type string of pubkey to `/cosmos.crypto.secp256k1.PubKey` and change the timestamp to `uint64`. The `provider_consensus_state` is exactly the `consensus_state` of the client state. The modified json may like this:

```json
    "provider_client_state": {
        "sequence": "1",
        "is_frozen": false,
        "consensus_state": {
            "public_key": {
                "@type": "/cosmos.crypto.secp256k1.PubKey",
                "key": "Ak8Np2EtL9lyOEGS3FASEDfZqmyYds/HWEtwUiqGBRi9"
            },
            "diversifier": "near",
            "timestamp": "1697661590804910403"
        }
    },
    "provider_consensus_state": {
        "public_key": {
            "@type": "/cosmos.crypto.secp256k1.PubKey",
            "key": "Ak8Np2EtL9lyOEGS3FASEDfZqmyYds/HWEtwUiqGBRi9"
        },
        "diversifier": "near",
        "timestamp": "1697661590804910403"
    },
```

Modify the `initial_val_set` of `genesis.json` with actual validators. For example:

```json
    "initial_val_set": [
        {
            "pub_key": {
                "ed25519": "lriei60AvqKy1VPOTQzm2Ka8MMxZEEwvONJtZrtBCSU="
            },
            "power": "20"
        }
    ],
```

> By using Oyster single node, you need to use the key in `~/.gm/<node name>/config/priv_validator_key.json` as the public key of the validator.

Then we can try to reset and restart the appchain and continue to the following steps.

### Create IBC connection between appchain and near ibc contract

```bash
hermes create connection --a-chain near-0 --a-client 07-tendermint-0 --b-client 06-solomachine-0 > hermes.log 2>&1 | tail -f hermes.log
```

### Create IBC channel between appchain and near ibc contract

```bash
hermes create channel --a-chain oct-cosmos-1-0 --a-connection connection-0 --a-port consumer --b-port provider --order ordered > hermes.log 2>&1 | tail -f hermes.log
```

> Note that, the channel handshake must be initiated from the appchain side.

### Start relaying messages

Once successfully created the channel, we can start relaying messages between appchain and near ibc contract.

```bash
hermes start
```

### Send initial validator set to appchain to confirm the connection

The following command will send the latest validator set (which is the validator set 0 at this time) to appchain.

```bash
near call $ANCHOR_ACCOUNT_ID send_vsc_packet_to_appchain '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
```

We can check the transactions in the NEAR explorer to confirm that the VSC packet has been successfully sent to the appchain. Additionally, consumer packets with `NotifyRewardsPacketData` will be processed by the near ibc contract periodically and successfully.

### Transfer reward token to appchain anchor account

For distributing reward of corresponding appchain, we need to transfer a certain amount of reward token to the appchain anchor account. Here is an example for cli command:

```bash
near call $REWARD_TOKEN ft_transfer_call '{"receiver_id":"oct-cosmos-1.registry.test_oct.testnet","amount":"15000000000000000000","memo":null,"msg":""}' --accountId riversyang.testnet --depositYocto 1 --gas 200000000000000
```

> The transfer must be done through `ft_transfer_call` function. Otherwise, the deposited reward tokens will NOT be accounted.

If the locked balance of reward token is not enough, the distribution request will be recorded in the appchain anchor contract. The distribution request can be checked by the following view function:

```bash
near view $ANCHOR_ACCOUNT_ID get_pending_rewards ''
```
