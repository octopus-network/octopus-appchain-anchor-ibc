use crate::*;
use near_sdk::PromiseResult;

#[ext_contract(ext_reward_token_callbacks)]
pub trait RewardTokenCallbacks {
    /// Callback function for `ft_transfer_call` of reward token contract
    fn ft_transfer_call_callback(&mut self, deposit_msg: FtTransferMessage, amount: U128);
}

#[near_bindgen]
impl RewardTokenCallbacks for AppchainAnchor {
    //
    fn ft_transfer_call_callback(&mut self, deposit_msg: FtTransferMessage, amount: U128) {
        near_sdk::assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(bytes) => {
                let accepted_amount: U128 = near_sdk::serde_json::from_slice(&bytes).unwrap();
                if accepted_amount.0 == amount.0 {
                    self.locked_reward_token_amount -= amount.0;
                    let max_gas = Gas::ONE_TERA.mul(10);
                    self.pending_rewards.remove_first(max_gas);
                } else {
                    if accepted_amount.0 > 0 {
                        let mut reward_distribution = self.pending_rewards.get_first().unwrap();
                        if accepted_amount.0 > amount.0 {
                            panic!(
                                "Accepted amount {} is larger than the transfered amount {}.",
                                accepted_amount.0, amount.0
                            );
                        }
                        reward_distribution.amount = U128(amount.0 - accepted_amount.0);
                        self.locked_reward_token_amount -= accepted_amount.0;
                        self.pending_rewards.update_first(&reward_distribution);
                    }
                    log!(
                        "Not all reward tokens are accepted by LPOS market contract for {:?}:\
                        transfered amount: {}, accepted amount: {}",
                        deposit_msg,
                        amount.0,
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
}
