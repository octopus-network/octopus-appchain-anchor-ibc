#!/bin/bash
set -e
#
# export NEAR_CLI_TESTNET_RPC_SERVER_URL=https://near-testnet.infura.io/v3/4f80a04e6eb2437a9ed20cb874e10d55
# export NEAR_CLI_TESTNET_RPC_SERVER_URL=https://public-rpc.blockpi.io/http/near-testnet
export NEAR_ENV=testnet
export APPCHAIN_ID=$1
export REGISTRY_ACCOUNT_ID=registry.test_oct.testnet
export ANCHOR_ACCOUNT_ID=$APPCHAIN_ID'.'$REGISTRY_ACCOUNT_ID
#
#
#
# near deploy --accountId $ANCHOR_ACCOUNT_ID --initFunction 'new' --initArgs '{"restaking_base_contract":"contract-5.restaking-base.testnet","lpos_market_contract":"contract-5.lpos-market.testnet","near_ibc_contract":"v9.nearibc.testnet","reward_token_contract":"oct.beta_oct_relay.testnet"}' --wasmFile res/appchain_anchor_ibc.wasm
#
# near call oct.beta_oct_relay.testnet storage_deposit '{"account_id":"oct-cosmos-1.registry.test_oct.testnet","registration_only":null}' --accountId my-account.testnet --deposit 0.0125
#
# near call oct.beta_oct_relay.testnet ft_transfer_call '{"receiver_id":"oct-cosmos-1.registry.test_oct.testnet","amount":"1000000000000000000","memo":null,"msg":""}' --accountId riversyang.testnet --depositYocto 1 --gas 200000000000000
#
# near deploy --accountId $ANCHOR_ACCOUNT_ID --initFunction 'migrate_state' --initArgs '{}' --wasmFile res/appchain_anchor_ibc.wasm
#
#
#
# near deploy --accountId $ANCHOR_ACCOUNT_ID --wasmFile res/appchain_anchor_ibc.wasm
#
# near call $ANCHOR_ACCOUNT_ID migrate_state '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID fetch_validator_set_from_restaking_base '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID create_client_for_appchain '{"initial_height":{"revision_number":0,"revision_height":1},"trusting_period":"1209600","unbonding_period":"1814400","max_clock_drift":"600","upgrade_path":[]}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID change_era_reward '{"era_reward":"1000000000"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID go_live '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID send_vsc_packet_to_appchain '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID change_era_reward '{"era_reward":"1000000000000000000"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID distribute_pending_rewards '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
#
#
# near call $ANCHOR_ACCOUNT_ID change_appchain_registry '{"appchain_registry":"registry.test_oct.testnet"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID change_near_ibc_contract '{"near_ibc_contract":"v8.nearibc.testnet"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID clear_pending_rewards '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID update_locked_reward_token_balance '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID change_vsc_packet_timeout_interval '{"interval_secs":"432000"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID change_min_validator_staking_amount '{"amount_in_near":"10"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
# near call $ANCHOR_ACCOUNT_ID send_vsc_packet_with_removing_addresses '{"removing_addresses":["ed25519:I0+S4rvAXWm/J9l4fbLx6kbW/hrNl8uYMEzheue1xQw=","ed25519:uAv/z2JvVkD2Ut3SRcQZ2oGQcw5lj4BvX8adDWo/8Yg="]}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
#
#
# ./tmp/testnet_view_cli.sh $1
