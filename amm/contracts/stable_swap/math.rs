use amm_helpers::math::casted_mul;
use ink::prelude::{vec, vec::Vec};
use primitive_types::U256;
use traits::MathError;

use crate::fees::Fees;

/// Max number of iterations for curve computation using Newton–Raphson method
pub const MAX_ITERATIONS: u8 = 255;

/// Computes stable swap invariant (D)
pub fn compute_d(amounts: &Vec<u128>, amp_coef: u128) -> Result<U256, MathError> {
    // SUM{x_i}
    let amount_sum = amounts.iter().try_fold(U256::from(0), |acc, &amount| {
        acc.checked_add(amount.into())
            .ok_or(MathError::AddOverflow(1))
    })?;
    if amount_sum == 0.into() {
        Ok(0.into())
    } else {
        let n = amounts.len() as u32;
        // A * n^n
        let ann: U256 = casted_mul(
            amp_coef,
            n.checked_pow(n).ok_or(MathError::MulOverflow(1))?.into(),
        );
        // A * n^n * SUM{x_i}
        let ann_sum = ann
            .checked_mul(amount_sum)
            .ok_or(MathError::MulOverflow(2))?;
        // A * n^n - 1
        let ann_sub_one = ann
            .checked_sub(1.into())
            .ok_or(MathError::SubUnderflow(1))?;
        // n + 1
        let n_add_one = n.checked_add(1).ok_or(MathError::AddOverflow(1))?;
        let mut d = amount_sum;
        // Computes next D unitl satisfying precision is reached
        for _ in 0..MAX_ITERATIONS {
            let d_next = compute_d_next(d, n, amounts, ann_sum, ann_sub_one, n_add_one)?;
            if d_next > d {
                if d_next.checked_sub(d).ok_or(MathError::SubUnderflow(2))? <= 1.into() {
                    return Ok(d);
                }
            } else if d.checked_sub(d_next).ok_or(MathError::SubUnderflow(3))? <= 1.into() {
                return Ok(d);
            }
            d = d_next;
        }
        Ok(d)
    }
}

fn compute_d_next(
    d_prev: U256,
    n: u32,
    amounts: &Vec<u128>,
    ann_sum: U256,
    ann_sub_one: U256,
    n_add_one: u32,
) -> Result<U256, MathError> {
    let mut d_prod = d_prev;
    // d_prod = ... * [d_prev / (x_(i) * n)] * ...
    // where i in (0,n)
    for amount in amounts {
        d_prod = d_prod
            .checked_mul(d_prev)
            .ok_or(MathError::MulOverflow(1))?
            .checked_div(
                amount
                    .checked_mul(n.into())
                    .ok_or(MathError::MulOverflow(2))?
                    .into(),
            )
            .ok_or(MathError::DivByZero(1))?;
    }
    let numerator = d_prev
        .checked_mul(
            d_prod
                .checked_mul(n.into())
                .ok_or(MathError::MulOverflow(3))?
                .checked_add(ann_sum)
                .ok_or(MathError::AddOverflow(1))?,
        )
        .ok_or(MathError::MulOverflow(4))?;
    let denominator = d_prev
        .checked_mul(ann_sub_one)
        .ok_or(MathError::MulOverflow(5))?
        .checked_add(
            d_prod
                .checked_mul(n_add_one.into())
                .ok_or(MathError::MulOverflow(6))?,
        )
        .ok_or(MathError::AddOverflow(2))?;
    numerator
        .checked_div(denominator)
        .ok_or(MathError::DivByZero(2))
}

/// Returns new reserve of `y` tokens
/// given new reserve of `x` tokens
///
/// NOTICE: it does not check if `token_x_id` != `token_y_id` and if tokens' `id`s are out of bounds
pub fn compute_y(
    new_reserve_x: u128,
    reserves: &Vec<u128>,
    token_x_id: usize,
    token_y_id: usize,
    amp_coef: u128,
) -> Result<u128, MathError> {
    let n = reserves.len() as u32;
    let ann: U256 = casted_mul(
        amp_coef,
        n.checked_pow(n).ok_or(MathError::MulOverflow(1))?.into(),
    );
    let d: U256 = compute_d(reserves, amp_coef)?;

    let mut c = d
        .checked_mul(d)
        .ok_or(MathError::MulOverflow(2))?
        .checked_div(new_reserve_x.into())
        .ok_or(MathError::DivByZero(1))?;
    let mut reservers_sum: U256 = new_reserve_x.into();
    // reserves_sum = ... + x_(i') + ...
    // c1 = ... * d / x_(i') * ... * d
    // where  i' in (0,n) AND i' != token_y_id
    for (idx, &reserve) in reserves.iter().enumerate() {
        if idx != token_x_id && idx != token_y_id {
            reservers_sum = reservers_sum
                .checked_add(reserve.into())
                .ok_or(MathError::AddOverflow(1))?;
            c = c
                .checked_mul(d)
                .ok_or(MathError::MulOverflow(3))?
                .checked_div(reserve.into())
                .ok_or(MathError::DivByZero(2))?;
        }
    }
    // c = c_1 * d / (A * n^2n)
    c = c
        .checked_mul(d)
        .ok_or(MathError::MulOverflow(4))?
        .checked_div(
            ann.checked_mul((n).checked_pow(n).ok_or(MathError::MulOverflow(5))?.into())
                .ok_or(MathError::MulOverflow(6))?,
        )
        .ok_or(MathError::DivByZero(3))?;
    // reserves_sum + d / ( A * n^n)
    let b: U256 = d
        .checked_div(ann)
        .ok_or(MathError::DivByZero(4))?
        .checked_add(reservers_sum)
        .ok_or(MathError::AddOverflow(2))?; // d will be subtracted later

    let mut y_prev = d;
    let mut y = y_prev;
    for _ in 0..MAX_ITERATIONS {
        y = compute_y_next(y_prev, b, c, d)?;
        if y > y_prev {
            if y.checked_sub(y_prev).ok_or(MathError::SubUnderflow(2))? <= 1.into() {
                return Ok(y.as_u128());
            }
        } else if y_prev.checked_sub(y).ok_or(MathError::SubUnderflow(3))? <= 1.into() {
            return Ok(y.as_u128());
        }
        y_prev = y;
    }
    Ok(y.as_u128())
}

fn compute_y_next(y_prev: U256, b: U256, c: U256, d: U256) -> Result<U256, MathError> {
    let numerator = y_prev
        .checked_pow(2.into())
        .ok_or(MathError::MulOverflow(1))?
        .checked_add(c)
        .ok_or(MathError::AddOverflow(1))?;
    let denominator = y_prev
        .checked_mul(2.into())
        .ok_or(MathError::MulOverflow(2))?
        .checked_add(b)
        .ok_or(MathError::AddOverflow(2))?
        .checked_sub(d)
        .ok_or(MathError::SubUnderflow(1))?;
    numerator
        .checked_div(denominator)
        .ok_or(MathError::DivByZero(1))
}

/// Encodes all results of swapping from a source token to a destination token.
#[derive(Debug)]
pub struct SwapResult {
    /// New amount of source token.
    pub new_source_amount: u128,
    /// New amount of destination token (with fees applied).
    pub new_destination_amount: u128,
    /// If token_in_amount is known:
    ///     Amount of destination token swapped (with fees applied).
    /// If token_out_amount is known:
    ///     Amount of source token swapped (with fees applied).
    pub amount_swapped: u128,
    /// Admin fee for the swap (part of the `fee`).
    pub admin_fee: u128,
    /// Fee for the swap (applied to token_out).
    pub fee: u128,
}

/// Compute SwapResult after an exchange given `amount_in` of the `token_in_id`
/// panics if token ids are out of bounds
/// NOTICE: it does not check if `token_in_id` != `token_out_id`
pub fn swap_to(
    token_in_idx: usize,
    token_in_amount: u128,
    token_out_idx: usize,
    current_reserves: &Vec<u128>,
    fees: &Fees,
    amp_coef: u128,
    admin_fee: bool, // flag if admin fee should be accounted
) -> Result<SwapResult, MathError> {
    let y = compute_y(
        token_in_amount
            .checked_add(current_reserves[token_in_idx])
            .ok_or(MathError::AddOverflow(1))?,
        current_reserves,
        token_in_idx,
        token_out_idx,
        amp_coef,
    )?;
    // sub 1 in case there are any rounding errors
    // https://github.com/curvefi/curve-contract/blob/b0bbf77f8f93c9c5f4e415bce9cd71f0cdee960e/contracts/pool-templates/base/SwapTemplateBase.vy#L466
    let dy = current_reserves[token_out_idx]
        .checked_sub(y)
        .ok_or(MathError::SubUnderflow(1))?
        .checked_sub(1)
        .ok_or(MathError::SubUnderflow(2))?;
    // fees are applied to "token_out" amount
    let trade_fee = fees.trade_fee_from_gross(dy)?;
    let amount_swapped = dy
        .checked_sub(trade_fee)
        .ok_or(MathError::SubUnderflow(3))?;

    let mut new_destination_amount = current_reserves[token_out_idx]
        .checked_sub(amount_swapped)
        .ok_or(MathError::SubUnderflow(4))?;
    let optional_admin_fee = if admin_fee {
        let admin_fee_computed = fees.admin_trade_fee(trade_fee)?;
        new_destination_amount = new_destination_amount
            .checked_sub(admin_fee_computed)
            .ok_or(MathError::SubUnderflow(5))?;
        admin_fee_computed
    } else {
        0
    };
    let new_source_amount = current_reserves[token_in_idx]
        .checked_add(token_in_amount)
        .ok_or(MathError::AddOverflow(1))?;

    Ok(SwapResult {
        new_source_amount,
        new_destination_amount,
        amount_swapped,
        admin_fee: optional_admin_fee,
        fee: trade_fee,
    })
}

/// Compute SwapResult after an exchange given `amount_out` of the `token_out_id`
/// panics if token ids are out of bounds
/// NOTICE: it does not check if `token_in_id` != `token_out_id`
pub fn swap_from(
    token_out_idx: usize,
    token_out_amount: u128, // Net amount (w/o fee)
    token_in_idx: usize,
    current_reserves: &Vec<u128>,
    fees: &Fees,
    amp_coef: u128,
    admin_fee: bool,
) -> Result<SwapResult, MathError> {
    // fees are applied to "token_out" amount
    let trade_fee = fees.trade_fee_from_net(token_out_amount)?;
    let token_out_amount_plus_fee = token_out_amount
        .checked_add(trade_fee)
        .ok_or(MathError::AddOverflow(1))?;

    let y = compute_y(
        current_reserves[token_out_idx]
            .checked_sub(token_out_amount_plus_fee)
            .ok_or(MathError::SubUnderflow(1))?,
        current_reserves,
        token_out_idx,
        token_in_idx,
        amp_coef,
    )?;
    // add 1 in case there are any rounding errors
    // https://github.com/curvefi/curve-contract/blob/b0bbf77f8f93c9c5f4e415bce9cd71f0cdee960e/contracts/pool-templates/base/SwapTemplateBase.vy#L466
    let dy: u128 = y
        .checked_sub(current_reserves[token_in_idx])
        .ok_or(MathError::SubUnderflow(2))?
        .checked_add(1)
        .ok_or(MathError::AddOverflow(1))?;

    let mut new_destination_amount = current_reserves[token_out_idx]
        .checked_sub(token_out_amount)
        .ok_or(MathError::SubUnderflow(4))?;
    let optional_admin_fee = if admin_fee {
        let admin_fee_computed = fees.admin_trade_fee(trade_fee)?;
        new_destination_amount = new_destination_amount
            .checked_sub(admin_fee_computed)
            .ok_or(MathError::SubUnderflow(5))?;
        admin_fee_computed
    } else {
        0
    };
    let new_source_amount = current_reserves[token_in_idx]
        .checked_add(dy)
        .ok_or(MathError::AddOverflow(1))?;

    Ok(SwapResult {
        new_source_amount,
        new_destination_amount,
        amount_swapped: dy,
        admin_fee: optional_admin_fee,
        fee: trade_fee,
    })
}

/// Compute the amount of LP tokens to mint after a deposit
/// return <lp_amount_to_mint, lp_fees_part>
pub fn compute_lp_amount_for_deposit(
    deposit_amounts: &Vec<u128>,
    old_reserves: &Vec<u128>,
    pool_token_supply: u128,
    fees: Option<&Fees>,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    if pool_token_supply == 0 {
        for &amount in deposit_amounts {
            if amount == 0 {
                return Err(MathError::DivByZero(42));
            }
        }
        Ok((
            compute_d(deposit_amounts, amp_coef)?
                .try_into()
                .map_err(|_| MathError::CastOverflow(1))?,
            0,
        ))
    } else {
        // Initial invariant
        let d_0 = compute_d(old_reserves, amp_coef)?;
        let n_coins = old_reserves.len();
        let mut new_reserves = vec![0_u128; n_coins];
        for (index, value) in deposit_amounts.iter().enumerate() {
            new_reserves[index] = old_reserves[index]
                .checked_add(*value)
                .ok_or(MathError::AddOverflow(1))?;
        }
        // Invariant after change
        let d_1 = compute_d(&new_reserves, amp_coef)?;
        if let Some(_fees) = fees {
            // Recalculate the invariant accounting for fees
            for i in 0..new_reserves.len() {
                let ideal_reserve: u128 = d_1
                    .checked_mul(old_reserves[i].into())
                    .ok_or(MathError::MulOverflow(1))?
                    .checked_div(d_0)
                    .ok_or(MathError::DivByZero(1))?
                    .try_into()
                    .map_err(|_| MathError::CastOverflow(1))?;
                let difference = if ideal_reserve > new_reserves[i] {
                    ideal_reserve
                        .checked_sub(new_reserves[i])
                        .ok_or(MathError::SubUnderflow(1))?
                } else {
                    new_reserves[i]
                        .checked_sub(ideal_reserve)
                        .ok_or(MathError::SubUnderflow(2))?
                };
                let fee = _fees.normalized_trade_fee(n_coins as u32, difference)?;
                new_reserves[i] = new_reserves[i]
                    .checked_sub(fee)
                    .ok_or(MathError::SubUnderflow(3))?;
            }
            let d_2: U256 = compute_d(&new_reserves, amp_coef)?;
            let mint_shares: u128 = U256::from(pool_token_supply)
                .checked_mul(d_2.checked_sub(d_0).ok_or(MathError::SubUnderflow(4))?)
                .ok_or(MathError::MulOverflow(2))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(2))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(1))?;

            let diff_shares: u128 = U256::from(pool_token_supply)
                .checked_mul(d_1.checked_sub(d_0).ok_or(MathError::SubUnderflow(5))?)
                .ok_or(MathError::MulOverflow(3))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(3))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(2))?;
            // d1 > d2 > d0,
            // (d2-d0) => mint_shares (charged fee),
            // (d1-d0) => diff_shares (without fee),
            // (d1-d2) => fee part,
            // diff_shares = mint_shares + fee part
            Ok((
                mint_shares,
                diff_shares
                    .checked_sub(mint_shares)
                    .ok_or(MathError::SubUnderflow(6))?,
            ))
        } else {
            // Calc without fees
            let mint_shares: u128 = U256::from(pool_token_supply)
                .checked_mul(d_1.checked_sub(d_0).ok_or(MathError::SubUnderflow(7))?)
                .ok_or(MathError::MulOverflow(4))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(4))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(3))?;
            // d1 > d0,
            // (d1-d0) => mint_shares
            Ok((mint_shares, 0))
        }
    }
}

/// Compute the ideal amounts of deposits for lp mint
/// return <deposit_amounts, new_reserves>
pub fn compute_deposit_amounts_for_lp(
    lp_amount: u128,
    old_reserves: &Vec<u128>,
    pool_token_supply: u128,
) -> Result<(Vec<u128>, Vec<u128>), MathError> {
    let n_coins = old_reserves.len();
    let mut amounts = Vec::new();
    let mut new_reserves = old_reserves.clone();
    for i in 0..n_coins {
        amounts.push(
            U256::from(old_reserves[i])
                .checked_mul(lp_amount.into())
                .ok_or(MathError::MulOverflow(1))?
                .checked_div(pool_token_supply.into())
                .ok_or(MathError::DivByZero(1))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(1))?,
        );
        new_reserves[i] = new_reserves[i]
            .checked_add(*amounts.last().unwrap())
            .ok_or(MathError::SubUnderflow(1))?;
    }
    Ok((amounts, new_reserves))
}

/// given token_out user want get and total tokens in pool and lp token supply,
/// return <lp_amount_to_burn, lp_fees_part>
/// all amounts are in c_amount (comparable amount)
pub fn compute_lp_amount_for_withdraw(
    withdraw_amounts: &[u128],
    old_reserves: &Vec<u128>,
    pool_token_supply: u128,
    fees: Option<&Fees>,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    let n_coins = old_reserves.len();
    // Initial invariant, D0
    let d_0 = compute_d(old_reserves, amp_coef)?;

    // real invariant after withdraw, D1
    let mut new_reserves = vec![0_u128; n_coins];
    for (index, value) in withdraw_amounts.iter().enumerate() {
        new_reserves[index] = old_reserves[index]
            .checked_sub(*value)
            .ok_or(MathError::SubUnderflow(1))?;
    }
    let d_1 = compute_d(&new_reserves, amp_coef)?;

    // Recalculate the invariant accounting for fees
    if let Some(_fees) = fees {
        for i in 0..new_reserves.len() {
            let ideal_u128 = d_1
                .checked_mul(old_reserves[i].into())
                .ok_or(MathError::MulOverflow(1))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(1))?
                .as_u128();
            let difference = if ideal_u128 > new_reserves[i] {
                ideal_u128
                    .checked_sub(new_reserves[i])
                    .ok_or(MathError::SubUnderflow(2))?
            } else {
                new_reserves[i]
                    .checked_sub(ideal_u128)
                    .ok_or(MathError::SubUnderflow(3))?
            };
            let fee = _fees.normalized_trade_fee(n_coins as u32, difference)?;
            // new_u128 is for calculation D2, the one with fee charged
            new_reserves[i] = new_reserves[i]
                .checked_sub(fee)
                .ok_or(MathError::SubUnderflow(4))?;
        }
        let d_2 = compute_d(&new_reserves, amp_coef)?;
        // d0 > d1 > d2,
        // (d0-d2) => burn_shares (plus fee),
        // (d0-d1) => diff_shares (without fee),
        // (d1-d2) => fee part,
        // burn_shares = diff_shares + fee part

        let burn_shares = U256::from(pool_token_supply)
            .checked_mul(d_0.checked_sub(d_2).ok_or(MathError::SubUnderflow(5))?)
            .ok_or(MathError::MulOverflow(4))?
            .checked_div(d_0)
            .ok_or(MathError::DivByZero(3))?
            .as_u128();
        let diff_shares = U256::from(pool_token_supply)
            .checked_mul(d_0.checked_sub(d_1).ok_or(MathError::SubUnderflow(6))?)
            .ok_or(MathError::MulOverflow(5))?
            .checked_div(d_0)
            .ok_or(MathError::DivByZero(4))?
            .as_u128();

        Ok((
            burn_shares,
            burn_shares
                .checked_sub(diff_shares)
                .ok_or(MathError::SubUnderflow(7))?,
        ))
    } else {
        let burn_shares = U256::from(pool_token_supply)
            .checked_mul(d_0.checked_sub(d_1).ok_or(MathError::SubUnderflow(5))?)
            .ok_or(MathError::MulOverflow(4))?
            .checked_div(d_0)
            .ok_or(MathError::DivByZero(3))?
            .as_u128();
        Ok((burn_shares, 0))
    }
}

/// compute amounts to withdraw for lp tokens (no fee)
/// returns <amounts_to_withdraw, new_reserves>
pub fn compute_withdraw_amounts_for_lp(
    lp_amount: u128,
    old_reserves: &Vec<u128>,
    pool_token_supply: u128,
) -> Result<(Vec<u128>, Vec<u128>), MathError> {
    let n_coins = old_reserves.len();
    let mut amounts = Vec::new();
    let mut new_reserves = old_reserves.clone();
    for i in 0..n_coins {
        amounts.push(
            U256::from(old_reserves[i])
                .checked_mul(lp_amount.into())
                .ok_or(MathError::MulOverflow(1))?
                .checked_div(pool_token_supply.into())
                .ok_or(MathError::DivByZero(1))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(1))?,
        );
        new_reserves[i] = new_reserves[i]
            .checked_sub(*amounts.last().unwrap())
            .ok_or(MathError::SubUnderflow(1))?;
    }
    Ok((amounts, new_reserves))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d_computation_high_amp_coef() {
        let amp_coef: u128 = 1_000_000_000_000;
        let reserve_0: u128 = 400_000_000_000;
        let reserve_1: u128 = 500_000_000_000;
        let d = compute_d(&Vec::from([reserve_0, reserve_1]), amp_coef)
            .unwrap_or_else(|err| panic!("Should compute: {err:?}"));
        assert_eq!(
            d,
            (reserve_0 + reserve_1).into(),
            "Invariant should be equal constant sum invariant"
        )
    }

    #[test]
    fn d_computation_low_amp_coef() {
        let amp_coef: u128 = 1;
        let reserve_0: u128 = 400_000_000_000;
        let reserve_1: u128 = 500_000_000_000;
        let d = compute_d(&Vec::from([reserve_0, reserve_1]), amp_coef)
            .unwrap_or_else(|err| panic!("Should compute: {err:?}"));
        assert!(
            d < (reserve_0 + reserve_1).into(),
            "Invariant should be less than const sum invariant"
        );
        let prod_d = casted_mul(reserve_0, reserve_1).integer_sqrt() * 2;
        assert!(
            d > prod_d,
            "Invariant should be greater than const prod invariant"
        );
    }

    #[test]
    fn y_computation_high_amp_coef() {
        let amp_coef: u128 = 1_000_000_000_000;
        let reserve_0: u128 = 500_000_000_000;
        let reserve_1: u128 = 500_000_000_000;
        let reserve_delta: u128 = 40_000_000_000;
        let reserve_0_after = reserve_0 - reserve_delta;
        let reserve_1_after = compute_y(
            reserve_0_after,
            &Vec::from([reserve_0, reserve_1]),
            0,
            1,
            amp_coef,
        )
        .unwrap_or_else(|err| panic!("Should compute y. Err: {err:?}"));
        assert_eq!(
            reserve_1_after,
            reserve_1 + reserve_delta,
            "Reserve change should be linear"
        );
    }

    #[test]
    fn y_computation_low_amp_coef() {
        let amp_coef: u128 = 1;
        let reserve_0: u128 = 400_000_000_000;
        let reserve_1: u128 = 500_000_000_000;
        let reserve_delta: u128 = 40_000_000_000;
        let reserve_0_after = reserve_0 - reserve_delta;
        let reserve_1_after = compute_y(
            reserve_0_after,
            &Vec::from([reserve_0, reserve_1]),
            0,
            1,
            amp_coef,
        )
        .unwrap_or_else(|err| panic!("Should compute y. Err: {err:?}"));
        assert!(
            reserve_1_after > reserve_1 + reserve_delta,
            "Reserve destination reserve change should be greater than source reserve"
        );
        let const_prod_y = (reserve_1 * (reserve_0 + reserve_delta)) / reserve_0;
        assert!(
            const_prod_y > reserve_1_after,
            "Destination reserve change should be less than in const prod swap"
        );
    }

    #[test]
    fn swap_to_computation_no_fees() {
        let amp_coef: u128 = 1000;
        let fees = Fees::zero();
        let reserves: Vec<u128> = vec![100000000000, 100000000000];
        let token_in = 10000000000;
        // amounts from https://github.com/ref-finance/ref-contracts/blob/be5c0e33465c13a05dab6e5e9ff9f8af414e16a7/ref-exchange/src/stable_swap/mod.rs#L744
        let expect_token_out = 9999495232;
        let swap_result = swap_to(0, token_in, 1, &reserves, &fees, amp_coef, false)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(
            swap_result.amount_swapped, expect_token_out,
            "Incorrect swap ammount"
        );
        assert_eq!(
            swap_result.new_source_amount,
            reserves[0] + token_in,
            "Incorrect new source amount"
        );
        assert_eq!(
            swap_result.new_destination_amount,
            reserves[1] - expect_token_out,
            "Incorrect new destination amount"
        );
    }

    #[test]
    fn swap_from_computation_no_fees() {
        let amp_coef: u128 = 1000;
        let fees = Fees::zero();
        let reserves: Vec<u128> = vec![100000000000, 100000000000];
        let token_out = 9999495232;
        let expect_token_in = 10000000000;
        let swap_result = swap_from(0, token_out, 1, &reserves, &fees, amp_coef, true)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(
            swap_result.amount_swapped, expect_token_in,
            "Incorrect swap ammount"
        );
        assert_eq!(
            swap_result.new_source_amount,
            reserves[1] + expect_token_in,
            "Incorrect new source amount"
        );
        assert_eq!(
            swap_result.new_destination_amount,
            reserves[0] - token_out,
            "Incorrect new destination amount"
        );
    }

    #[test]
    fn swap_to_computation_with_fees() {
        let amp_coef: u128 = 1000;
        let fees = Fees::new(1000, 0); // 10% fee
        let reserves: Vec<u128> = vec![100000000000, 100000000000];
        let token_in = 10000000000;
        let expect_token_out = 9999495232;
        let expect_fee = expect_token_out / 10;
        let expect_token_out_minus_fee = expect_token_out - expect_fee;
        let swap_result = swap_to(0, token_in, 1, &reserves, &fees, amp_coef, false)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(
            swap_result.amount_swapped, expect_token_out_minus_fee,
            "Incorrect swap ammount"
        );
        assert_eq!(swap_result.fee, expect_fee, "Incorrect total fee ammount");
        assert_eq!(swap_result.admin_fee, 0, "Incorrect admin fee ammount");
        assert_eq!(
            swap_result.new_source_amount,
            reserves[0] + token_in,
            "Incorrect new source amount"
        );
        assert_eq!(
            swap_result.new_destination_amount,
            reserves[1] - expect_token_out_minus_fee,
            "Incorrect new destination amount"
        );
    }

    #[test]
    fn swap_from_computation_with_fees() {
        let amp_coef: u128 = 1000;
        let fees = Fees::new(1000, 0); // 10% fee
        let reserves: Vec<u128> = vec![100000000000, 100000000000];
        let token_out = 9999495232;
        let expect_fee: u128 = 9999495232 / 10;
        let token_out_minus_expect_fee = token_out - expect_fee;
        let expect_token_in = 10000000000;
        let swap_result = swap_from(
            0,
            token_out_minus_expect_fee,
            1,
            &reserves,
            &fees,
            amp_coef,
            true,
        )
        .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(
            swap_result.amount_swapped, expect_token_in,
            "Incorrect swap ammount"
        );
        assert_eq!(swap_result.fee, expect_fee, "Incorrect total fee ammount");
        assert_eq!(swap_result.admin_fee, 0, "Incorrect admin fee ammount");
        assert_eq!(
            swap_result.new_source_amount,
            reserves[0] + expect_token_in,
            "Incorrect new source amount"
        );
        assert_eq!(
            swap_result.new_destination_amount,
            reserves[1] - token_out_minus_expect_fee,
            "Incorrect new destination amount"
        );
    }

    #[test]
    fn swap_to_from_computation() {
        let amp_coef: u128 = 1000;
        let fees = Fees::new(1000, 0);
        // let fees = Fees::zero();
        let reserves: Vec<u128> = vec![110000000000, 91000454291];
        let token_0_in: u128 = 10000000000;
        let swap_to_result = swap_to(0, token_0_in, 1, &reserves, &fees, amp_coef, false)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        let swap_from_result = swap_from(
            1,
            swap_to_result.amount_swapped,
            0,
            &reserves,
            &fees,
            amp_coef,
            false,
        )
        .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(
            swap_from_result.amount_swapped, token_0_in,
            "Incorrect swap amount"
        );
        assert_eq!(
            swap_from_result.fee, swap_to_result.fee,
            "Incorrect fee amount"
        );
    }

    #[test]
    fn swap_from_to_computation() {
        let amp_coef: u128 = 1000;
        let fees = Fees::new(1000, 0);
        // let fees = Fees::zero();
        let reserves: Vec<u128> = vec![110000000000, 91000454291];
        let token_0_out: u128 = 10000000000;
        let swap_from_result = swap_from(0, token_0_out, 1, &reserves, &fees, amp_coef, false)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        let swap_to_result = swap_to(
            1,
            swap_from_result.amount_swapped,
            0,
            &reserves,
            &fees,
            amp_coef,
            false,
        )
        .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(
            swap_to_result.amount_swapped, token_0_out,
            "Incorrect swap amount"
        );
        assert_eq!(
            swap_to_result.fee, swap_from_result.fee,
            "Incorrect fee amount"
        );
    }

    #[test]
    fn withdraw_liquidity_by_share_and_by_amounts_equality() {
        let amp_coef: u128 = 85;
        let fees = Fees::new(1000, 2000); // 10% fee
        let reserves: Vec<u128> = Vec::from([500_000_000_000, 500_000_000_000]);
        let token_supply = compute_d(&reserves, amp_coef).unwrap().as_u128();
        let share = token_supply / 100; // 1%
        let (withdraw_amounts_by_share, _) =
            compute_withdraw_amounts_for_lp(share, &reserves, token_supply)
                .unwrap_or_else(|_| panic!("Should work"));
        let (share_by_withdraw_amounts, fee_part) = compute_lp_amount_for_withdraw(
            &withdraw_amounts_by_share,
            &reserves,
            token_supply,
            Some(&fees),
            amp_coef,
        )
        .unwrap_or_else(|_| panic!("Should work"));
        assert_eq!(fee_part, 0, "Fee is should be 0");
        assert_eq!(
            share_by_withdraw_amounts, share,
            "Share amounts should match"
        );
    }

    #[test]
    fn deposit_liquidity_by_share_and_by_amounts_equality() {
        let amp_coef: u128 = 85;
        let fees = Fees::new(1000, 2000); // 10% fee
        let reserves: Vec<u128> = Vec::from([500_000_000_000, 500_000_000_000]);
        let token_supply = compute_d(&reserves, amp_coef).unwrap().as_u128();
        let amounts: Vec<u128> = Vec::from([50_000_000, 50_000_000]);
        let (share, fee_part) =
            compute_lp_amount_for_deposit(&amounts, &reserves, token_supply, Some(&fees), amp_coef)
                .unwrap_or_else(|_| panic!("Should mint liquidity"));
        assert_eq!(fee_part, 0, "Fee is should be 0");
        let reserves_a = vec![reserves[0] + amounts[0], reserves[1] + amounts[1]];
        let (deposit_amounts, reserves_b) =
            compute_deposit_amounts_for_lp(share, &reserves, token_supply)
                .unwrap_or_else(|_| panic!("Should mint liquidity"));
        assert_eq!(amounts, deposit_amounts, "Deposit amounts differ.");
        assert_eq!(reserves_a, reserves_b, "Reserves should match");
    }
    #[test]
    fn biggie() {
        let res = 10u128.checked_pow(12).unwrap();
        assert!(res
            .checked_mul(1_000_000_000_000_000_000_000_000_00)
            .is_some())
    }
}
