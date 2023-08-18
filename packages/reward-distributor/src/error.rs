use cosmwasm_std::StdError;
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
}
