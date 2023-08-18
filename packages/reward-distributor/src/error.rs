use cosmwasm_std::{StdError, Uint128};
use cw_ownable::OwnershipError;

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error(transparent)]
    CwDex(#[from] cw_dex::CwDexError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("No rewards to distribute")]
    NoRewardsToDistribute {},

    #[error("Can only distribute once per block")]
    CanOnlyDistributeOncePerBlock {},

    #[error(
        "Rewards have not started. Rewards start time: {start_time}. Current time: {current_time}"
    )]
    RewardsNotStarted { start_time: u64, current_time: u64 },

    #[error("Insufficient vault token balance. Vault token balance: {vault_token_balance}. Redeem amount: {redeem_amount}")]
    InsufficientVaultTokenBalance {
        vault_token_balance: Uint128,
        redeem_amount: Uint128,
    },
}
