use ink::prelude::vec::Vec;
use ink::primitives::AccountId;
use ink::LangError;
use psp22::PSP22Error;

use crate::MathError;

#[ink::trait_definition]
pub trait StablePoolView {
    /// Returns list of tokens in the pool.
    #[ink(message)]
    fn tokens(&self) -> Vec<AccountId>;

    /// Returns list of reserves in comparable amounts.
    #[ink(message)]
    fn reserves(&self) -> Vec<u128>;

    /// Returns current value of amplification coefficient.
    #[ink(message)]
    fn amp_coef(&self) -> Result<u128, StablePoolError>;

    /// Calculate swap amount of token_out
    /// given token_in amount
    /// Returns (amount_out, fee)
    /// fee is applied to token_out
    #[ink(message)]
    fn get_swap_amount_out(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        token_in_amount: u128,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Calculate required amount of token_in
    /// given swap token_out amount
    /// Returns (amount_in, fee)
    /// fee is applied to token_out
    #[ink(message)]
    fn get_swap_amount_in(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        token_out_amount: u128,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Calculate how many lp tokens will be minted
    /// given deposit `amounts`.
    /// Returns (lp_amount, fee)
    #[ink(message)]
    fn get_mint_liquidity_for_amounts(
        &self,
        amounts: Vec<u128>,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Calculate ideal deposit amounts required
    /// to mint `liquidity` amount of lp tokens
    /// Returns required deposit amounts
    #[ink(message)]
    fn get_amounts_for_liquidity_mint(&self, liquidity: u128)
        -> Result<Vec<u128>, StablePoolError>;

    /// Calculate how many lp tokens will be burned
    /// given withdraw `amounts`.
    /// Returns (lp_amount, fee)
    #[ink(message)]
    fn get_burn_liquidity_for_amounts(
        &self,
        amounts: Vec<u128>,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Calculate ideal withdraw amounts for
    /// burning `liquidity` amount of lp tokens
    /// Returns withdraw amounts
    #[ink(message)]
    fn get_amounts_for_liquidity_burn(&self, liquidity: u128)
        -> Result<Vec<u128>, StablePoolError>;
}

#[ink::trait_definition]
pub trait StablePool {
    /// Mints LP tokens to `to` account from imbalanced `amounts`.
    /// `to` account must allow enough spending allowance of underlying tokens
    /// for this contract.
    /// Returns an error if the minted LP tokens amount is less
    /// than `min_share_amount`.
    /// Returns <(minted_shares, fee_part),_>
    #[ink(message)]
    fn add_liquidity(
        &mut self,
        min_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Burns LP tokens and withdraws underlying tokens to `to` account
    /// in imbalanced `amounts`.
    /// Returns <(burned_share_amount, fee_part),_>
    #[ink(message)]
    fn remove_liquidity(
        &mut self,
        max_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Swaps token_in to token_out.
    /// Swapped tokens are transferred to the `to` account.
    /// caller account must allow enough spending allowance of token_in
    /// for this contract.
    /// Returns an error if swapped token_out amount is less than
    /// `min_token_out_amount`.
    /// Returns <(token_out_amount, fee_amount),_>
    #[ink(message)]
    fn swap(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        token_in_amount: u128,
        min_token_out_amount: u128,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Swaps the excess (balance - reserve) of token_in to token_out.
    /// Swapped tokens are transferred to the `to` account.
    /// Returns an error if swapped token_out amount is less than
    /// `min_token_out_amount`.
    /// Returns <(token_out_amount, fee_amount),_>
    #[ink(message)]
    fn swap_excess(
        &mut self,
        token_in_id: AccountId,
        token_out_id: AccountId,
        min_token_out_amount: u128,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Intializes amp_coef gradual change to `target_amp_coef` over `ramp_duration` milisecs
    /// @dev This method should be resticted to owner/admin
    #[ink(message)]
    fn ramp_amp_coef(
        &mut self,
        target_amp_coef: u128,
        ramp_duration: u64,
    ) -> Result<(), StablePoolError>;

    #[ink(message)]
    fn set_owner(&mut self, new_owner: AccountId) -> Result<(), StablePoolError>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum StablePoolError {
    MathError(MathError),
    PSP22Error(PSP22Error),
    LangError(LangError),
    InvalidTokenId(AccountId),
    IdenticalTokenId,
    IncorrectAmountsCount,
    InsufficientLiquidityMinted,
    InsufficientLiquidityBurned,
    InsufficientOutputAmount,
    InsufficientInputAmount,
    InsufficientLiquidity,
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
