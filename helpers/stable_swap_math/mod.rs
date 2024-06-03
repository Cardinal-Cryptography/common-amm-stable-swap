pub mod fees;

use crate::math::{casted_mul, MathError};
use ink::prelude::vec::Vec;
use primitive_types::U256;

use fees::Fees;

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
        let n_add_one = n.checked_add(1).ok_or(MathError::AddOverflow(2))?;
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
            .ok_or(MathError::MulOverflow(3))?
            .checked_div(
                amount
                    .checked_mul(n.into())
                    .ok_or(MathError::MulOverflow(4))?
                    .into(),
            )
            .ok_or(MathError::DivByZero(1))?;
    }
    let numerator = d_prev
        .checked_mul(
            d_prod
                .checked_mul(n.into())
                .ok_or(MathError::MulOverflow(5))?
                .checked_add(ann_sum)
                .ok_or(MathError::AddOverflow(3))?,
        )
        .ok_or(MathError::MulOverflow(6))?;
    let denominator = d_prev
        .checked_mul(ann_sub_one)
        .ok_or(MathError::MulOverflow(7))?
        .checked_add(
            d_prod
                .checked_mul(n_add_one.into())
                .ok_or(MathError::MulOverflow(8))?,
        )
        .ok_or(MathError::AddOverflow(4))?;
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
        n.checked_pow(n).ok_or(MathError::MulOverflow(9))?.into(),
    );
    let d: U256 = compute_d(reserves, amp_coef)?;

    let mut c = d
        .checked_mul(d)
        .ok_or(MathError::MulOverflow(10))?
        .checked_div(new_reserve_x.into())
        .ok_or(MathError::DivByZero(3))?;
    let mut reservers_sum: U256 = new_reserve_x.into();
    // reserves_sum = ... + x_(i') + ...
    // c1 = ... * d / x_(i') * ... * d
    // where  i' in (0,n) AND i' != token_y_id
    for (idx, &reserve) in reserves.iter().enumerate() {
        if idx != token_x_id && idx != token_y_id {
            reservers_sum = reservers_sum
                .checked_add(reserve.into())
                .ok_or(MathError::AddOverflow(5))?;
            c = c
                .checked_mul(d)
                .ok_or(MathError::MulOverflow(11))?
                .checked_div(reserve.into())
                .ok_or(MathError::DivByZero(4))?;
        }
    }
    // c = c_1 * d / (A * n^2n)
    c = c
        .checked_mul(d)
        .ok_or(MathError::MulOverflow(12))?
        .checked_div(
            ann.checked_mul((n).checked_pow(n).ok_or(MathError::MulOverflow(13))?.into())
                .ok_or(MathError::MulOverflow(14))?,
        )
        .ok_or(MathError::DivByZero(5))?;
    // reserves_sum + d / ( A * n^n)
    let b: U256 = d
        .checked_div(ann)
        .ok_or(MathError::DivByZero(6))?
        .checked_add(reservers_sum)
        .ok_or(MathError::AddOverflow(6))?; // d will be subtracted later

    let mut y_prev = d;
    let mut y = y_prev;
    for _ in 0..MAX_ITERATIONS {
        y = compute_y_next(y_prev, b, c, d)?;
        if y > y_prev {
            if y.checked_sub(y_prev).ok_or(MathError::SubUnderflow(4))? <= 1.into() {
                return Ok(y.as_u128());
            }
        } else if y_prev.checked_sub(y).ok_or(MathError::SubUnderflow(5))? <= 1.into() {
            return Ok(y.as_u128());
        }
        y_prev = y;
    }
    Ok(y.as_u128())
}

fn compute_y_next(y_prev: U256, b: U256, c: U256, d: U256) -> Result<U256, MathError> {
    let numerator = y_prev
        .checked_pow(2.into())
        .ok_or(MathError::MulOverflow(15))?
        .checked_add(c)
        .ok_or(MathError::AddOverflow(7))?;
    let denominator = y_prev
        .checked_mul(2.into())
        .ok_or(MathError::MulOverflow(16))?
        .checked_add(b)
        .ok_or(MathError::AddOverflow(8))?
        .checked_sub(d)
        .ok_or(MathError::SubUnderflow(6))?;
    numerator
        .checked_div(denominator)
        .ok_or(MathError::DivByZero(7))
}

/// Compute SwapResult after an exchange given `amount_in` of the `token_in_id`.
/// panics if token ids are out of bounds.
/// NOTICE: it does not check if `token_in_id` != `token_out_id`.
/// Returns (amount_out, fee_amount)
pub fn swap_to(
    token_in_idx: usize,
    token_in_amount: u128,
    token_out_idx: usize,
    current_reserves: &Vec<u128>,
    fees: &Fees,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    let y = compute_y(
        token_in_amount
            .checked_add(current_reserves[token_in_idx])
            .ok_or(MathError::AddOverflow(9))?,
        current_reserves,
        token_in_idx,
        token_out_idx,
        amp_coef,
    )?;
    // sub 1 in case there are any rounding errors
    // https://github.com/curvefi/curve-contract/blob/b0bbf77f8f93c9c5f4e415bce9cd71f0cdee960e/contracts/pool-templates/base/SwapTemplateBase.vy#L466
    let dy = current_reserves[token_out_idx]
        .checked_sub(y)
        .ok_or(MathError::SubUnderflow(7))?
        .checked_sub(1)
        .ok_or(MathError::SubUnderflow(8))?;
    // fees are applied to "token_out" amount
    let fee = fees.trade_fee_from_gross(dy)?;
    let amount_swapped = dy.checked_sub(fee).ok_or(MathError::SubUnderflow(9))?;

    Ok((amount_swapped, fee))
}

/// Compute SwapResult after an exchange given `amount_out` of the `token_out_id`
/// panics if token ids are out of bounds
/// NOTICE: it does not check if `token_in_id` != `token_out_id`
/// /// Returns (amount_in, fee_amount)
pub fn swap_from(
    token_out_idx: usize,
    token_out_amount: u128, // Net amount (w/o fee)
    token_in_idx: usize,
    current_reserves: &Vec<u128>,
    fees: &Fees,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    // fees are applied to "token_out" amount
    let fee = fees.trade_fee_from_net(token_out_amount)?;
    let token_out_amount_plus_fee = token_out_amount
        .checked_add(fee)
        .ok_or(MathError::AddOverflow(11))?;

    let y = compute_y(
        current_reserves[token_out_idx]
            .checked_sub(token_out_amount_plus_fee)
            .ok_or(MathError::SubUnderflow(12))?,
        current_reserves,
        token_out_idx,
        token_in_idx,
        amp_coef,
    )?;
    // add 1 in case there are any rounding errors
    // https://github.com/curvefi/curve-contract/blob/b0bbf77f8f93c9c5f4e415bce9cd71f0cdee960e/contracts/pool-templates/base/SwapTemplateBase.vy#L466
    let dy: u128 = y
        .checked_sub(current_reserves[token_in_idx])
        .ok_or(MathError::SubUnderflow(13))?
        .checked_add(1)
        .ok_or(MathError::AddOverflow(12))?;

    Ok((dy, fee))
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
        if deposit_amounts.contains(&0) {
            return Err(MathError::DivByZero(8));
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
        let mut new_reserves = old_reserves
            .iter()
            .zip(deposit_amounts.iter())
            .map(|(reserve, &amount)| {
                reserve
                    .checked_add(amount)
                    .ok_or(MathError::AddOverflow(14))
            })
            .collect::<Result<Vec<u128>, MathError>>()?;
        // Invariant after change
        let d_1 = compute_d(&new_reserves, amp_coef)?;
        if let Some(_fees) = fees {
            // Recalculate the invariant accounting for fees
            for i in 0..new_reserves.len() {
                let ideal_reserve: u128 = d_1
                    .checked_mul(old_reserves[i].into())
                    .ok_or(MathError::MulOverflow(17))?
                    .checked_div(d_0)
                    .ok_or(MathError::DivByZero(9))?
                    .try_into()
                    .map_err(|_| MathError::CastOverflow(2))?;
                let difference = if ideal_reserve > new_reserves[i] {
                    ideal_reserve
                        .checked_sub(new_reserves[i])
                        .ok_or(MathError::SubUnderflow(16))?
                } else {
                    new_reserves[i]
                        .checked_sub(ideal_reserve)
                        .ok_or(MathError::SubUnderflow(17))?
                };
                let fee = _fees.normalized_trade_fee(n_coins as u32, difference)?;
                new_reserves[i] = new_reserves[i]
                    .checked_sub(fee)
                    .ok_or(MathError::SubUnderflow(18))?;
            }
            let d_2: U256 = compute_d(&new_reserves, amp_coef)?;
            let mint_shares: u128 = U256::from(pool_token_supply)
                .checked_mul(d_2.checked_sub(d_0).ok_or(MathError::SubUnderflow(19))?)
                .ok_or(MathError::MulOverflow(18))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(10))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(3))?;

            let diff_shares: u128 = U256::from(pool_token_supply)
                .checked_mul(d_1.checked_sub(d_0).ok_or(MathError::SubUnderflow(20))?)
                .ok_or(MathError::MulOverflow(19))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(11))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(4))?;
            // d1 > d2 > d0,
            // (d2-d0) => mint_shares (charged fee),
            // (d1-d0) => diff_shares (without fee),
            // (d1-d2) => fee part,
            // diff_shares = mint_shares + fee part
            Ok((
                mint_shares,
                diff_shares
                    .checked_sub(mint_shares)
                    .ok_or(MathError::SubUnderflow(21))?,
            ))
        } else {
            // Calc without fees
            let mint_shares: u128 = U256::from(pool_token_supply)
                .checked_mul(d_1.checked_sub(d_0).ok_or(MathError::SubUnderflow(22))?)
                .ok_or(MathError::MulOverflow(20))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(12))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(5))?;
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
    let mut amounts = Vec::new();
    let mut new_reserves = Vec::new();
    for i in 0..old_reserves.len() {
        amounts.push(
            U256::from(old_reserves[i])
                .checked_mul(lp_amount.into())
                .ok_or(MathError::MulOverflow(21))?
                .checked_div(pool_token_supply.into())
                .ok_or(MathError::DivByZero(13))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(6))?,
        );
        new_reserves.push(
            old_reserves[i]
                .checked_add(*amounts.last().unwrap())
                .ok_or(MathError::SubUnderflow(23))?,
        );
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
    let mut new_reserves = old_reserves
        .iter()
        .zip(withdraw_amounts.iter())
        .map(|(reserve, &amount)| {
            reserve
                .checked_sub(amount)
                .ok_or(MathError::AddOverflow(14))
        })
        .collect::<Result<Vec<u128>, MathError>>()?;
    let d_1 = compute_d(&new_reserves, amp_coef)?;

    // Recalculate the invariant accounting for fees
    if let Some(_fees) = fees {
        for i in 0..new_reserves.len() {
            let ideal_u128 = d_1
                .checked_mul(old_reserves[i].into())
                .ok_or(MathError::MulOverflow(22))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(14))?
                .as_u128();
            let difference = if ideal_u128 > new_reserves[i] {
                ideal_u128
                    .checked_sub(new_reserves[i])
                    .ok_or(MathError::SubUnderflow(25))?
            } else {
                new_reserves[i]
                    .checked_sub(ideal_u128)
                    .ok_or(MathError::SubUnderflow(26))?
            };
            let fee = _fees.normalized_trade_fee(n_coins as u32, difference)?;
            // new_u128 is for calculation D2, the one with fee charged
            new_reserves[i] = new_reserves[i]
                .checked_sub(fee)
                .ok_or(MathError::SubUnderflow(27))?;
        }
        let d_2 = compute_d(&new_reserves, amp_coef)?;
        // d0 > d1 > d2,
        // (d0-d2) => burn_shares (plus fee),
        // (d0-d1) => diff_shares (without fee),
        // (d1-d2) => fee part,
        // burn_shares = diff_shares + fee part

        let burn_shares = U256::from(pool_token_supply)
            .checked_mul(d_0.checked_sub(d_2).ok_or(MathError::SubUnderflow(28))?)
            .ok_or(MathError::MulOverflow(23))?
            .checked_div(d_0)
            .ok_or(MathError::DivByZero(15))?
            .as_u128();
        let diff_shares = U256::from(pool_token_supply)
            .checked_mul(d_0.checked_sub(d_1).ok_or(MathError::SubUnderflow(29))?)
            .ok_or(MathError::MulOverflow(24))?
            .checked_div(d_0)
            .ok_or(MathError::DivByZero(16))?
            .as_u128();

        Ok((
            burn_shares,
            burn_shares
                .checked_sub(diff_shares)
                .ok_or(MathError::SubUnderflow(30))?,
        ))
    } else {
        let burn_shares = U256::from(pool_token_supply)
            .checked_mul(d_0.checked_sub(d_1).ok_or(MathError::SubUnderflow(31))?)
            .ok_or(MathError::MulOverflow(25))?
            .checked_div(d_0)
            .ok_or(MathError::DivByZero(17))?
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
    let mut amounts = Vec::new();
    let mut new_reserves = Vec::new();
    for i in 0..old_reserves.len() {
        amounts.push(
            U256::from(old_reserves[i])
                .checked_mul(lp_amount.into())
                .ok_or(MathError::MulOverflow(26))?
                .checked_div(pool_token_supply.into())
                .ok_or(MathError::DivByZero(18))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(7))?,
        );
        new_reserves.push(
            old_reserves[i]
                .checked_sub(*amounts.last().unwrap())
                .ok_or(MathError::SubUnderflow(32))?,
        );
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
            "Destination reserve change should be greater than in const sum swap"
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
        let (amount_out, fee) = swap_to(0, token_in, 1, &reserves, &fees, amp_coef)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(amount_out, expect_token_out, "Incorrect swap ammount");
        assert_eq!(fee, 0, "Fee should nbe 0");
    }

    #[test]
    fn swap_from_computation_no_fees() {
        let amp_coef: u128 = 1000;
        let fees = Fees::zero();
        let reserves: Vec<u128> = vec![100000000000, 100000000000];
        let token_out = 9999495232;
        let expect_token_in = 10000000000;
        let (amount_in, fee) = swap_from(0, token_out, 1, &reserves, &fees, amp_coef)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(amount_in, expect_token_in, "Incorrect swap ammount");
        assert_eq!(fee, 0, "Fee should nbe 0");
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
        let (amount_out, fee) = swap_to(0, token_in, 1, &reserves, &fees, amp_coef)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(
            amount_out, expect_token_out_minus_fee,
            "Incorrect swap ammount"
        );
        assert_eq!(fee, expect_fee, "Incorrect total fee ammount");
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
        let (amount_in, fee) =
            swap_from(0, token_out_minus_expect_fee, 1, &reserves, &fees, amp_coef)
                .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(amount_in, expect_token_in, "Incorrect swap ammount");
        assert_eq!(fee, expect_fee, "Incorrect total fee ammount");
    }

    #[test]
    fn swap_to_from_computation() {
        let amp_coef: u128 = 1000;
        let fees = Fees::new(2137, 0);
        let reserves: Vec<u128> = vec![12341234123412341234, 5343245543253432435];
        let token_0_in: u128 = 62463425433;
        let (amount_out, fee_out) = swap_to(0, token_0_in, 1, &reserves, &fees, amp_coef)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        let (amount_in, fee_in) = swap_from(1, amount_out, 0, &reserves, &fees, amp_coef)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(amount_in, token_0_in, "Incorrect swap amount");
        assert_eq!(fee_out, fee_in, "Incorrect fee amount");
    }

    #[test]
    fn swap_from_to_computation() {
        let amp_coef: u128 = 1000;
        let fees = Fees::new(2137, 0);
        let reserves: Vec<u128> = vec![12341234123412341234, 5343245543253432435];
        let token_0_out: u128 = 62463425433;

        let (amount_in, fee_in) = swap_from(0, token_0_out, 1, &reserves, &fees, amp_coef)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        let (amount_out, fee_out) = swap_to(1, amount_in, 0, &reserves, &fees, amp_coef)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(amount_out, token_0_out, "Incorrect swap amount");
        assert_eq!(fee_in, fee_out, "Incorrect fee amount");
    }

    #[test]
    fn swap_to_from_computation_edge_case() {
        let amp_coef: u128 = 80;
        let fees = Fees::new(1000, 0);
        let reserves: Vec<u128> = vec![100000000000, 100000000000];
        let token_0_in: u128 = 1000000;
        let (amount_out, fee_out) = swap_to(1, token_0_in, 0, &reserves, &fees, amp_coef)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        let (amount_in, fee_in) = swap_from(0, amount_out, 1, &reserves, &fees, amp_coef)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(amount_in, token_0_in, "Incorrect swap amount");
        // amount_in:   (1000001, 1000000)
        assert_eq!(fee_in, fee_out, "Incorrect fee amount");
        // fee:         (100000, 99999)
    }

    #[test]
    fn swap_from_to_computation_edge_case() {
        let amp_coef: u128 = 80;
        let fees = Fees::new(1000, 0);
        let reserves: Vec<u128> = vec![100000000000, 100000000000];
        let token_0_out: u128 = 1000000;
        let (amount_in, fee_in) = swap_from(0, token_0_out, 1, &reserves, &fees, amp_coef)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        let (amount_out, fee_out) = swap_to(1, amount_in, 0, &reserves, &fees, amp_coef)
            .unwrap_or_else(|_| panic!("Should return SwapResult"));
        assert_eq!(amount_out, token_0_out, "Incorrect swap amount");
        assert_eq!(fee_in, fee_out, "Incorrect fee amount");
    }

    #[test]
    fn withdraw_liquidity_by_share_and_by_amounts_equality_1() {
        let amp_coef: u128 = 85;
        let fees = Fees::new(2137, 0); // 10% fee
        let reserves: Vec<u128> = Vec::from([500_000_000_000, 500_000_000_000]);
        let token_supply = compute_d(&reserves, amp_coef).unwrap().as_u128();
        let share = token_supply / 20; // 5%
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
        assert_eq!(fee_part, 0, "Fee should be 0");
        assert_eq!(
            share_by_withdraw_amounts, share,
            "Share amounts should match"
        );
    }

    #[test]
    fn deposit_liquidity_by_share_and_by_amounts_equality_1() {
        let amp_coef: u128 = 85;
        let fees = Fees::new(2137, 0); // 10% fee
        let reserves: Vec<u128> = Vec::from([500_000_000_000, 500_000_000_000]);
        let token_supply = compute_d(&reserves, amp_coef).unwrap().as_u128();
        let share = token_supply / 20; // 5%
        let (deposit_amounts, reserves_b) =
            compute_deposit_amounts_for_lp(share, &reserves, token_supply)
                .unwrap_or_else(|_| panic!("Should mint liquidity"));
        let (share_by_deposit, fee_part) = compute_lp_amount_for_deposit(
            &deposit_amounts,
            &reserves,
            token_supply,
            Some(&fees),
            amp_coef,
        )
        .unwrap_or_else(|_| panic!("Should mint liquidity"));
        assert_eq!(fee_part, 0, "Fee should be 0");
        let reserves_a = vec![
            reserves[0] + deposit_amounts[0],
            reserves[1] + deposit_amounts[1],
        ];
        assert_eq!(share, share_by_deposit, "Deposit amounts differ.");
        assert_eq!(reserves_a, reserves_b, "Reserves should match");
    }

    #[test]
    fn withdraw_liquidity_by_share_and_by_amounts_equality_2() {
        let amp_coef: u128 = 85;
        let fees = Fees::new(2137, 0); // 10% fee
        let reserves: Vec<u128> = Vec::from([12341234123412341234, 5343245543253432435]);
        let token_supply = compute_d(&reserves, amp_coef).unwrap().as_u128();
        let share = token_supply / 20; // 5%
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
        assert_eq!(fee_part, 0, "Fee should be 0");
        assert_eq!(
            share_by_withdraw_amounts, share,
            "Share amounts should match"
        );
    }

    #[test]
    fn deposit_liquidity_by_share_and_by_amounts_equality_2() {
        let amp_coef: u128 = 85;
        let fees = Fees::new(2137, 0);
        let reserves: Vec<u128> = Vec::from([12341234123412341234, 5343245543253432435]);
        let token_supply = compute_d(&reserves, amp_coef).unwrap().as_u128();
        let share = token_supply / 20; // 5%
        let (deposit_amounts, reserves_b) =
            compute_deposit_amounts_for_lp(share, &reserves, token_supply)
                .unwrap_or_else(|_| panic!("Should mint liquidity"));
        let (share_by_deposit, fee_part) = compute_lp_amount_for_deposit(
            &deposit_amounts,
            &reserves,
            token_supply,
            Some(&fees),
            amp_coef,
        )
        .unwrap_or_else(|_| panic!("Should mint liquidity"));
        assert_eq!(fee_part, 0, "Fee should be 0");
        let reserves_a = vec![
            reserves[0] + deposit_amounts[0],
            reserves[1] + deposit_amounts[1],
        ];
        assert_eq!(reserves_a, reserves_b, "Reserves should match");
        assert_eq!(share, share_by_deposit, "Deposit amounts differ.");
    }
}
