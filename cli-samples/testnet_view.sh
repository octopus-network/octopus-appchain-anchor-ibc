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
# near state $ANCHOR_ACCOUNT_ID
#
# near view $ANCHOR_ACCOUNT_ID version
#
# near view $ANCHOR_ACCOUNT_ID get_chain_id
#
# near view $ANCHOR_ACCOUNT_ID get_anchor_settings
#
# near view $ANCHOR_ACCOUNT_ID get_anchor_status
#
# near view $ANCHOR_ACCOUNT_ID get_validator_set '{"index":"0"}'
#
# near view $ANCHOR_ACCOUNT_ID get_latest_validator_set
#
# near view $ANCHOR_ACCOUNT_ID get_pending_rewards
#
# near view $ANCHOR_ACCOUNT_ID get_pending_slash_packets
#
# near view oct.beta_oct_relay.testnet ft_balance_of '{"account_id":"oct-cosmos-1.registry.test_oct.testnet"}'
#
# near view oct.beta_oct_relay.testnet ft_balance_of '{"account_id":"contract-2.lpos-market.testnet"}'
