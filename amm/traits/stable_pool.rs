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

    /// Returns list of tokens reserves.
    #[ink(message)]
    fn reserves(&self) -> Vec<u128>;

    /// Returns current value of amplification coefficient.
    #[ink(message)]
    fn amp_coef(&self) -> u128;

    /// Calculate swap amount of token_out
    /// given token_in amount
    /// Returns (amount_out, fee)
    /// fee is applied to token_out
    #[ink(message)]
    fn get_swap_amount_out(
        &mut self,
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
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        token_out_amount: u128,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Calculate how many lp tokens will be minted
    /// given deposit `amounts`.
    /// Returns (lp_amount, fee)
    #[ink(message)]
    fn get_mint_liquidity_for_amounts(
        &mut self,
        amounts: Vec<u128>,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Calculate ideal deposit amounts required
    /// to mint `liquidity` amount of lp tokens
    /// Returns required deposit amounts
    #[ink(message)]
    fn get_amounts_for_liquidity_mint(
        &mut self,
        liquidity: u128,
    ) -> Result<Vec<u128>, StablePoolError>;

    /// Calculate how many lp tokens will be burned
    /// given withdraw `amounts`.
    /// Returns (lp_amount, fee)
    #[ink(message)]
    fn get_burn_liquidity_for_amounts(
        &mut self,
        amounts: Vec<u128>,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Calculate ideal withdraw amounts for
    /// burning `liquidity` amount of lp tokens
    /// Returns withdraw amounts
    #[ink(message)]
    fn get_amounts_for_liquidity_burn(
        &mut self,
        liquidity: u128,
    ) -> Result<Vec<u128>, StablePoolError>;
}

#[ink::trait_definition]
pub trait StablePool {
    /// Mints LP tokens to `to` account from imbalanced `amounts`.
    /// `to` account must allow enough spending allowance of underlying tokens
    /// for this contract.
    /// Returns an error if the minted LP tokens amount is less
    /// than `min_share_amount`.
    /// Returns (minted_shares, fee_part)
    #[ink(message)]
    fn add_liquidity(
        &mut self,
        min_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Burns LP tokens and withdraws underlying tokens to `to` account
    /// in imbalanced `amounts`.
    /// Returns (burned_share_amount, fee_part)
    #[ink(message)]
    fn remove_liquidity_by_amounts(
        &mut self,
        max_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Burns LP tokens and withdraws underlying tokens in balanced amounts to `to` account.
    /// Fails if any of the amounts received is less than in `min_amounts`.
    /// Returns ([amounts_by_tokens], fee_part)
    #[ink(message)]
    fn remove_liquidity_by_shares(
        &mut self,
        shares: u128,
        min_amounts: Vec<u128>,
        to: AccountId,
    ) -> Result<Vec<u128>, StablePoolError>;

    /// Swaps token_in to token_out.
    /// Swapped tokens are transferred to the `to` account.
    /// caller account must allow enough spending allowance of token_in
    /// for this contract.
    /// Returns an error if swapped token_out amount is less than
    /// `min_token_out_amount`.
    /// Returns (token_out_amount, fee_amount)
    #[ink(message)]
    fn swap_exact_in(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        token_in_amount: u128,
        min_token_out_amount: u128,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Swaps token_in to token_out.
    /// Swapped tokens are transferred to the `to` account.
    /// caller account must allow enough spending allowance of token_out
    /// for this contract.
    /// Returns an error if to get token_out_amount of token_out it is required
    /// to spend more than `max_token_in_amount` of token_in.
    /// Returns (token_in_amount, fee_amount)
    #[ink(message)]
    fn swap_exact_out(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        token_out_amount: u128,
        max_token_in_amount: u128,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Swaps excess reserve balance of `token_in` to `token_out`.
    ///
    /// Swapped tokens are transferred to the `to` account.
    /// Returns (amount_out, fees)
    #[ink(message)]
    fn swap(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        min_token_out_amount: u128,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    #[ink(message)]
    fn set_owner(&mut self, new_owner: AccountId) -> Result<(), StablePoolError>;

    #[ink(message)]
    fn set_fee_receiver(&mut self, fee_receiver: Option<AccountId>) -> Result<(), StablePoolError>;

    #[ink(message)]
    fn set_amp_coef(&mut self, amp_coef: u128) -> Result<(), StablePoolError>;

    #[ink(message)]
    fn force_update_rate(&mut self);
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
    InvalidAmpCoef,
    InsufficientLiquidityMinted,
    InsufficientLiquidityBurned,
    InsufficientOutputAmount,
    TooLargeInputAmount,
    InsufficientInputAmount,
    IncorrectTokenCount,
    TooLargeTokenDecimal,
    OnlyOwner,
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
