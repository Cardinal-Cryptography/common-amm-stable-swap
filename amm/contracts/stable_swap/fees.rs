use amm_helpers::math::casted_mul;
use traits::MathError;

pub const FEE_BPS_DENOM: u32 = 10_000;

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Fees {
    pub trade_fee_bps: u32,
    pub admin_fee_bps: u32,
}

impl Fees {
    pub fn new(total_fee_bps: u32, admin_fee_bps: u32) -> Self {
        Self {
            trade_fee_bps: total_fee_bps,
            admin_fee_bps,
        }
    }

    pub fn zero() -> Self {
        Self {
            trade_fee_bps: 0,
            admin_fee_bps: 0,
        }
    }

    pub fn trade_fee_from_gross(&self, amount: u128) -> Result<u128, MathError> {
        u128_ratio(amount, self.trade_fee_bps, FEE_BPS_DENOM)
    }

    pub fn trade_fee_from_net(&self, amount: u128) -> Result<u128, MathError> {
        u128_ratio(
            amount,
            self.trade_fee_bps,
            FEE_BPS_DENOM
                .checked_sub(self.trade_fee_bps)
                .ok_or(MathError::SubUnderflow(1))?,
        )
    }

    pub fn admin_trade_fee(&self, amount: u128) -> Result<u128, MathError> {
        u128_ratio(amount, self.admin_fee_bps, FEE_BPS_DENOM)
    }

    /// Used to normalize fee applid on difference amount with ideal u128, This logic is from
    /// https://github.com/ref-finance/ref-contracts/blob/main/ref-exchange/src/stable_swap/math.rs#L48
    /// https://github.com/saber-hq/stable-swap/blob/5db776fb0a41a0d1a23d46b99ef412ca7ccc5bf6/stable-swap-program/program/src/fees.rs#L73
    /// https://github.com/curvefi/curve-contract/blob/e5fb8c0e0bcd2fe2e03634135806c0f36b245511/tests/simulation.py#L124
    pub fn normalized_trade_fee(&self, num_coins: u32, amount: u128) -> Result<u128, MathError> {
        let adjusted_trade_fee = (self
            .trade_fee_bps
            .checked_mul(num_coins)
            .ok_or(MathError::MulOverflow(1)))?
        .checked_div(
            (num_coins.checked_sub(1).ok_or(MathError::SubUnderflow(1)))?
                .checked_mul(4)
                .ok_or(MathError::MulOverflow(2))?,
        )
        .ok_or(MathError::DivByZero(1))?;
        u128_ratio(amount, adjusted_trade_fee, FEE_BPS_DENOM)
    }
}

fn u128_ratio(amount: u128, num: u32, denom: u32) -> Result<u128, MathError> {
    casted_mul(amount, num.into())
        .checked_div(denom.into())
        .ok_or(MathError::DivByZero(1))?
        .try_into()
        .map_err(|_| MathError::CastOverflow(1))
}
