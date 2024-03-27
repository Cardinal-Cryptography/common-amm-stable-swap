use ink::prelude::vec::Vec;
use ink::primitives::AccountId;
use ink::LangError;
use psp22::PSP22Error;

use crate::MathError;

#[ink::trait_definition]
pub trait StablePool {
    /// Returns list of tokens in the pool.
    #[ink(message)]
    fn tokens(&self) -> Vec<AccountId>;

    /// Returns list of reserves in comparable amounts.
    #[ink(message)]
    fn reserves(&self) -> Vec<u128>;

    /// Mints liquidity to `to` account.
    /// The amount minted is equivalent to the excess of contract's balance and reserves.
    /// Return amount of minted shares
    #[ink(message)]
    fn mint_liquidity(&mut self, to: AccountId) -> Result<u128, StablePoolError>;

    /// Burns transferred liquidity based on specified `amounts`.
    /// Surplus of the LP tokens is transfered to `to` account.
    /// If `amounts` are `None` burns all tranferred liquidity.
    /// The tokens are withdrawn to `to` account.
    /// Returns amount of <burnt_shares,withdraw_amounts>.
    #[ink(message)]
    fn burn_liquidity(
        &mut self,
        to: AccountId,
        amounts: Option<Vec<u128>>,
    ) -> Result<(u128, Vec<u128>), StablePoolError>;

    /// Swaps received amount of `token_in_id` for some amount of `token_out_id`.
    #[ink(message)]
    fn swap(
        &mut self,
        token_in_id: u8,
        token_out_id: u8,
        to: AccountId,
    ) -> Result<(), StablePoolError>;

    /// Returns current value of amplification coefficient.
    #[ink(message)]
    fn amp_coef(&self) -> Result<u128, StablePoolError>;

    /// Intializes amp_coef gradual change to `target_amp_coef` over `ramp_duration` milisecs
    /// @dev This method should be resticted to owner/admin
    #[ink(message)]
    fn ramp_amp_coef(
        &mut self,
        target_amp_coef: u128,
        ramp_duration: u64,
    ) -> Result<(), StablePoolError>;

    #[ink(message)]
    fn set_admin(&mut self, new_admin: AccountId) -> Result<(), StablePoolError>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum StablePoolError {
    MathError(MathError),
    PSP22Error(PSP22Error),
    LangError(LangError),
    InvalidTokenId(u8),
    IdenticalTokenId,
    IncorrectAmountsCount,
    InsufficientLiquidityMinted,
    InsufficientLiquidityBurned,
    InsufficientOutputAmount,
    InsufficientLiquidity,
    InsufficientInputAmount,
    ReservesOverflow,
    IncorrectTokenCount,
    TokenDecimals,
    AmpCoefToHigh,
    AmpCoefToLow,
    AmpCoefChangeToLow,
    AmpCoefRampDurationToShort,
    OnlyAdmin,
}

impl From<PSP22Error> for StablePoolError {
    fn from(error: PSP22Error) -> Self {
        StablePoolError::PSP22Error(error)
    }
}

impl From<LangError> for StablePoolError {
    fn from(error: LangError) -> Self {
        StablePoolError::LangError(error)
    }
}

impl From<MathError> for StablePoolError {
    fn from(error: MathError) -> Self {
        StablePoolError::MathError(error)
    }
}
