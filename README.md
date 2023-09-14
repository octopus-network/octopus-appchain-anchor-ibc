# octopus-appchain-anchor-ibc

This contract acts as an anchor of a cosmos appchain (developed based on `cosmos-sdk`) of [Octopus Network](https://oct.network). It is in charge of managing the necessary data of the corresponding appchain in NEAR protocol, providing security and interoperability for the appchain. This contract needs to work together with [octopus-appchain-registry](https://github.com/octopus-network/octopus-appchain-registry), [restaking-base](https://github.com/octopus-network/restaking-base.git), [lpos-market](https://github.com/octopus-network/lpos_market.git) and [near-ibc](https://github.com/octopus-network/near-ibc.git).

Each cosmos appchain of Octopus Network will be bonded to an instance of this contract, which is deployed to a subaccount of [octopus-appchain-registry](https://github.com/octopus-network/octopus-appchain-registry).

Contents

* [Terminology](#terminology)
* [Function specification](#function-specification)
  * [Manage anchor settings](#manage-anchor-settings)
  * [Manage validator set](#manage-validator-set)
  * [Distribute rewards](#distribute-rewards)
  * [Manage appchain lifecycle](#manage-appchain-lifecycle)
* [Initial deployment](#initial-deployment)
* [Auditing](#auditing)
* [Testing](#testing)

## Terminology

* `owner`: The owner of this contract, controlled by Octopus Network.
* `appchain registry`: A NEAR contract which manage the lifecycle of appchains of Octopus Network, controlled by Octopus Network.
* `appchain state`: The state of an appchain, the state `staging`, `booting`, `active`, `frozen`, `broken` and `dead` will be managed in this contract.
* `validator`: A person who wants to act as a validator on the appchain corresponding to this contract. Validators are managed by `restaking-base` contract.
* `validator set`: A set of validators of the corresponding appchain. The validator set will be updated periodically.
* `reward token`: A NEP-141 token contract which is used to distribute rewards to validators.
* `era`: A certain period that the validator set of the corresponding appchain will be updated, and the rewards will be distributed to validators. It is defined by the appchain protocol.
* `anchor settings`: A set of settings for current appchain anchor.

## Function specification

Generally speaking, this contract has the following responsibilities:

* Manage the settings data related to corresponding appchain.
* Manage the lifecycle of corresponding appchain.
* Manage the validator set for corresponding appchain.
* Lock and distribute rewards for validators of corresponding appchain.

> This contract also acts as a sub-module of `Octopus LPOS` module of `near-ibc`, to handle several specific business functions.

This contract provides a set of view functions for querying the status of the contract and necessary data related to the above business functions.

The corresponding contracts, including `reward token`, `restaknig-base`, `lpos-market` and `near-ibc`, will be set in initialization of this contract.

The account id of this contract will be `<appchain id>.<octopus appchain registry account id>`.

### Manage anchor settings

This contract has a set of functions to manage the value of each field of `anchor settings`.

### Manage validator set

#### Manage public keys of validators

This contract has the following functions for `restaking-base` contract to call to manage the public keys of validators:

* `bond` - Set the public key of a certain validator.
* `change_key` - Change the public key of a certain validator.

Only `restaking-base` contract can call these functions.

#### Update validator set

A permissionless off-chain program will call `update_validator_set` function of this contract periodically, to pull the latest validator set information from `restaking-base` contract and compose the `validator set` of the corresponding appchain by adding the public keys of validators. And then this function will call `send_vsc_packet` function of `near-ibc` to send VSC packet to the corresponding appchain.

The general process of updating validator set is as follows:

![updating validator set](/images/update-validator-set.png)

The updating peroid of validator set is defined by the appchain protocol.

### Distribute rewards

This contract can receive `reward token` from preset `reward token` contract.

The `near-ibc` contract will call `distribute_rewards` function of this contract when it receives a certain packet which indicates that a certain `era` of the corresponding appchain has finished and needs to distribute the reward of the `era`. And only `near-ibc` contract can call `distrubute_rewards` function of this contract.

The `distribute_rewards` function will check the locked balance of `reward token`. If it's enough for distributing rewards, this function will call `ft_transfer_call` function of `reward token` contract to transfer the rewards to `lpos-market` contract with a certain message which indicates the `era` (sequence) and the `validator set`. Otherwise, this function will account the rewards, log the error and wait for the next `distribute_rewards` call.

![rewards distribution](/images/rewards-distribution.png)

The rewards amount for each `era` is set in `anchor settings`.

### Manage appchain lifecycle

The owner of appchain anchor can manually change the state of corresponding appchain. These actions need to check necessary conditions before changing the state of corresponding appchain. And after changing the state, this contract will call function `sync_state_of` of `appchain registry` contract to synchronize the state to `appchain registry`. (The `appchain registry` will ensure the caller account of this function is `<appchain_id>.<appchain registry account>`.)

## Initial deployment

TBD.

## Auditing

TBD.

## Testing

TBD.
