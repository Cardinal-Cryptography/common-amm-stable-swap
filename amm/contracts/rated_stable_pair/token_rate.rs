use amm_helpers::math::casted_mul;
use ink::{
    env::{
        call::{build_call, ExecutionInput, Selector},
        DefaultEnvironment,
    },
    prelude::vec::Vec,
    primitives::AccountId,
};
use traits::MathError;

const GET_SAZERO_RATE_SELECTOR: [u8; 4] = ink::selector_bytes!("get_azero_from_shares");
type GetSazeroRateReturnType = u128;

pub const SAZERO_DECIMALS: u8 = 12;
pub const AZERO_DECIMALS: u8 = 12;
pub const ONE_SAZERO: u128 = 10u128.pow(SAZERO_DECIMALS as u32);
pub const ONE_AZERO: u128 = 10u128.pow(AZERO_DECIMALS as u32);

// 24 h
pub const RATE_EXPIRE_TS: u64 = 86400000;

#[ink::storage_item]
#[derive(Debug, Clone)]
pub struct TokenRate {
    ///
    cached_token_rate: u128,
    ///
    last_token_rate_update_ts: u64,
    ///
    token_rate_contract: AccountId,
}

impl TokenRate {
    pub fn new(current_time: u64, token_rate_contract: AccountId) -> Self {
        let mut token_rate = Self {
            cached_token_rate: 0,
            last_token_rate_update_ts: current_time,
            token_rate_contract,
        };
        token_rate._update(current_time);
        token_rate
    }

    /// Updates cached rate with current rate if cache is older than `RATE_EXPIRE_TS`.
    pub fn update(&mut self, current_time: u64) {
        if self
            .last_token_rate_update_ts
            .checked_add(RATE_EXPIRE_TS)
            .unwrap()
            < current_time
        {
            self._update(current_time);
        }
    }

    /// Converts amount to rated amount if `id` is equal `0`.
    pub fn amount_to_rated_amount(&self, amount: u128, id: usize) -> Result<u128, MathError> {
        if id == 0 {
            Ok(casted_mul(amount, self.cached_token_rate)
                .checked_div(ONE_AZERO.into())
                .ok_or(MathError::DivByZero(201))?
                .as_u128())
        } else {
            Ok(amount)
        }
    }

    /// Converts rated amount to amount if `id` is equal `0`.
    pub fn rated_amount_to_amount(&self, rated_amount: u128, id: usize) -> Result<u128, MathError> {
        if id == 0 {
            Ok(casted_mul(rated_amount, ONE_AZERO)
                .checked_div(self.cached_token_rate.into())
                .ok_or(MathError::DivByZero(202))?
                .as_u128())
        } else {
            Ok(rated_amount)
        }
    }

    pub fn amounts_to_rated_amounts(&self, amounts: &Vec<u128>) -> Result<Vec<u128>, MathError> {
        let mut rated_amounts = amounts.clone();
        rated_amounts[0] = casted_mul(amounts[0], self.cached_token_rate)
            .checked_div(ONE_AZERO.into())
            .ok_or(MathError::DivByZero(203))?
            .as_u128();
        Ok(rated_amounts)
    }

    pub fn rated_amounts_to_amounts(
        &self,
        rated_amounts: &Vec<u128>,
    ) -> Result<Vec<u128>, MathError> {
        let mut amounts = rated_amounts.clone();
        amounts[0] = casted_mul(rated_amounts[0], ONE_AZERO)
            .checked_div(self.cached_token_rate.into())
            .ok_or(MathError::DivByZero(204))?
            .as_u128();
        Ok(amounts)
    }

    /// Gets current token rate through a cross-contract call.
    fn call_get_rate(&self) -> u128 {
        build_call::<DefaultEnvironment>()
            .call(self.token_rate_contract)
            .exec_input(
                ExecutionInput::new(Selector::new(GET_SAZERO_RATE_SELECTOR)).push_arg(ONE_SAZERO),
            )
            .returns::<GetSazeroRateReturnType>()
            .invoke()
    }

    fn _update(&mut self, current_time: u64) {
        self.cached_token_rate = self.call_get_rate();
        self.last_token_rate_update_ts = current_time;
    }
}
