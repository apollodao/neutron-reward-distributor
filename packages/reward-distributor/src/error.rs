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

    #[error("Insufficient vault token balance. Vault token balance: {vault_token_balance}. Redeem amount: {redeem_amount}")]
    InsufficientVaultTokenBalance {
        vault_token_balance: Uint128,
        redeem_amount: Uint128,
    },
}
