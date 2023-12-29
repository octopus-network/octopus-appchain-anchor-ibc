use crate::*;
use near_sdk::PromiseResult;

#[ext_contract(ext_reward_token_callbacks)]
pub trait RewardTokenCallbacks {
    /// Callback function for `ft_transfer_call` of reward token contract
    fn ft_transfer_call_callback(
        &mut self,
        deposit_msg: FtTransferMessage,
        reward_distribution: RewardDistribution,
        reward_distribution_index: U64,
    );
    ///
    fn ft_balance_of_callback(&mut self);
}

#[near_bindgen]
impl RewardTokenCallbacks for AppchainAnchor {
    //
    fn ft_transfer_call_callback(
        &mut self,
        deposit_msg: FtTransferMessage,
        reward_distribution: RewardDistribution,
        reward_distribution_index: U64,
    ) {
        near_sdk::assert_self();
        match env::promise_result(0) {
            PromiseResult::Successful(bytes) => {
                let accepted_amount: U128 = near_sdk::serde_json::from_slice(&bytes).unwrap();
                if accepted_amount.0 == reward_distribution.amount.0 {
                    let mut new_rd = reward_distribution.clone();
                    self.locked_reward_token_amount -= reward_distribution.amount.0;
                    new_rd.distributed = true;
                    self.pending_rewards
                        .update(&reward_distribution_index.0, &reward_distribution);
                } else {
                    if accepted_amount.0 > 0 {
                        let mut new_rd = reward_distribution.clone();
                        if accepted_amount.0 > reward_distribution.amount.0 {
                            panic!(
                                "Accepted amount {} is larger than the transfered amount {}.",
                                accepted_amount.0, reward_distribution.amount.0
                            );
                        }
                        new_rd.amount = U128(reward_distribution.amount.0 - accepted_amount.0);
                        self.locked_reward_token_amount -= accepted_amount.0;
                        self.pending_rewards
                            .update(&reward_distribution_index.0, &new_rd);
                    }
                    log!(
                        "Not all reward tokens are accepted by LPOS market contract for {:?}:\
                        transfered amount: {}, accepted amount: {}",
                        deposit_msg,
                        reward_distribution.amount.0,
                        accepted_amount.0
                    )
                }
            }
            PromiseResult::Failed => {
                log!(
                    "Failed to transfer reward tokens to LPOS market contract for {:?}.",
                    deposit_msg
                );
            }
        }
    }
    //
    fn ft_balance_of_callback(&mut self) {
        near_sdk::assert_self();
        match env::promise_result(0) {
            PromiseResult::Successful(bytes) => {
                let balance: U128 = near_sdk::serde_json::from_slice(&bytes).unwrap();
                self.locked_reward_token_amount = balance.0;
            }
            PromiseResult::Failed => {
                log!("Failed to get balance of reward token.");
            }
        }
    }
}
