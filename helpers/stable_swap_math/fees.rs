use crate::math::{casted_mul, MathError};

pub const FEE_BPS_DENOM: u16 = 10_000;

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Fees {
    pub trade_fee_bps: u16,
    pub protocol_fee_bps: u16,
}

impl Fees {
    // [OK]
    pub fn new(total_fee_bps: u16, protocol_fee_bps: u16) -> Option<Self> {
        if total_fee_bps > FEE_BPS_DENOM || protocol_fee_bps > FEE_BPS_DENOM {
            None
        } else {
            Some(Self {
                trade_fee_bps: total_fee_bps,
                protocol_fee_bps,
            })
        }
    }

    // [OK]
    pub fn zero() -> Self {
        Self {
            trade_fee_bps: 0,
            protocol_fee_bps: 0,
        }
    }

    // [OK]
    pub fn trade_fee_from_gross(&self, amount: u128) -> Result<u128, MathError> {
        u128_ratio(amount, self.trade_fee_bps, FEE_BPS_DENOM)
    }

    // [OK]
    pub fn trade_fee_from_net(&self, amount: u128) -> Result<u128, MathError> {
        u128_ratio(
            amount,
            self.trade_fee_bps,
            FEE_BPS_DENOM
                .checked_sub(self.trade_fee_bps)
                .ok_or(MathError::SubUnderflow(61))?,
        )
    }

    // [OK]
    pub fn protocol_trade_fee(&self, amount: u128) -> Result<u128, MathError> {
        u128_ratio(amount, self.protocol_fee_bps, FEE_BPS_DENOM)
    }

    /// Used to normalize fee applied on difference amount with ideal u128, This logic is from
    /// https://github.com/ref-finance/ref-contracts/blob/main/ref-exchange/src/stable_swap/math.rs#L48
    /// https://github.com/saber-hq/stable-swap/blob/5db776fb0a41a0d1a23d46b99ef412ca7ccc5bf6/stable-swap-program/program/src/fees.rs#L73
    /// https://github.com/curvefi/curve-contract/blob/e5fb8c0e0bcd2fe2e03634135806c0f36b245511/tests/simulation.py#L124
    // [OK] (logic matches the links, but who knows what it does...)
    pub fn normalized_trade_fee(&self, num_coins: u16, amount: u128) -> Result<u128, MathError> {
        let adjusted_trade_fee = (self
            .trade_fee_bps
            .checked_mul(num_coins)
            .ok_or(MathError::MulOverflow(61)))?
        .checked_div(
            (num_coins.checked_sub(1).ok_or(MathError::SubUnderflow(62)))?
                .checked_mul(4)
                .ok_or(MathError::MulOverflow(62))?,
        )
        .ok_or(MathError::DivByZero(61))?;
        u128_ratio(amount, adjusted_trade_fee, FEE_BPS_DENOM)
    }
}

// [OK]
fn u128_ratio(amount: u128, num: u16, denom: u16) -> Result<u128, MathError> {
    casted_mul(amount, num.into())
        .checked_div(denom.into())
        .ok_or(MathError::DivByZero(61))?
        .try_into()
        .map_err(|_| MathError::CastOverflow(61))
}

#[cfg(test)]
mod test {
    use super::Fees;

    #[test]
    fn test_net_gross_fee_equal() { // ok
        for i in 1..100 {
            let net = 880265296841066047 * i;
            let fees = Fees::new(30, 0).unwrap();
            let fee1 = fees.trade_fee_from_net(net).unwrap();

            let gross = net + fee1;
            let fee2 = fees.trade_fee_from_gross(gross).unwrap();

            assert_eq!(fee1, fee2);
        }
    }
}
