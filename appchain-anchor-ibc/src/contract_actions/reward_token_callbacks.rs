use crate::*;
use near_sdk::PromiseResult;

#[ext_contract(ext_reward_token_callbacks)]
pub trait RewardTokenCallbacks {
    /// Callback function for `ft_transfer_call` of reward token contract
    fn ft_transfer_call_callback(&mut self, deposit_msg: FtTransferMessage);
}

#[near_bindgen]
impl RewardTokenCallbacks for AppchainAnchor {
    //
    fn ft_transfer_call_callback(&mut self, deposit_msg: FtTransferMessage) {
        near_sdk::assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(bytes) => {
                let amount: U128 = near_sdk::serde_json::from_slice(&bytes).unwrap();
                if amount.0 == 0 {
                    let anchor_settings = self.anchor_settings.get().unwrap();
                    self.locked_reward_token_amount -= anchor_settings.era_reward.0;
                    let max_gas = Gas::ONE_TERA.mul(10);
                    self.pending_rewards.remove_first(max_gas);
                } else {
                    log!(
                        "Reward tokens are rejected by LPOS market contract for {:?}.",
                        deposit_msg
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
