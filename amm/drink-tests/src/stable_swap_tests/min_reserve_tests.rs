use super::*;
use drink::{runtime::MinimalRuntime, session::Session};

const USDT_DEC: u8 = 6;
const USDC_DEC: u8 = 6;
const DAI_DEC: u8 = 18;

fn test_min_initial_reserve(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    initial_reserves: Vec<u128>,
    expected_result: Result<(u128, u128), StablePoolError>,
) {
    let (stable_swap, _) = setup_stable_swap_with_tokens(
        session,
        token_decimals,
        initial_reserves.clone(),
        1000,
        25,
        6,
        BOB,
    );
    let res = stable_swap::add_liquidity(session, stable_swap, BOB, 1, initial_reserves, bob());
    assert_eq!(expected_result, res, "Unexpected result");
}

fn test_min_reserve_withdraw_liquidity(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    initial_reserves: Vec<u128>,
    withdraw_amounts: Vec<u128>,
    expected_result: Result<(u128, u128), StablePoolError>,
) {
    let (stable_swap, _) = setup_stable_swap_with_tokens(
        session,
        token_decimals,
        initial_reserves.clone(),
        1000,
        25,
        6,
        BOB,
    );
    _ = stable_swap::add_liquidity(session, stable_swap, BOB, 1, initial_reserves, bob());

    let res = stable_swap::remove_liquidity_by_amounts(
        session,
        stable_swap,
        BOB,
        u128::MAX, // allow inifite max share to burn
        withdraw_amounts,
        bob(),
    );
    assert_eq!(expected_result, res, "Unexpected result");
}

fn test_min_reserve_withdraw_liquidity_by_shares(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    initial_reserves: Vec<u128>,
    withdraw_shares: u128,
    expected_result: Result<Vec<u128>, StablePoolError>,
) {
    let (stable_swap, _) = setup_stable_swap_with_tokens(
        session,
        token_decimals,
        initial_reserves.clone(),
        1000,
        25,
        6,
        BOB,
    );
    _ = stable_swap::add_liquidity(session, stable_swap, BOB, 1, initial_reserves, bob());

    let res = stable_swap::remove_liquidity_by_shares(
        session,
        stable_swap,
        BOB,
        withdraw_shares,
        vec![1, 1, 1], // min withdraw amounts
        bob(),
    );
    assert_eq!(expected_result, res, "Unexpected result");
}

/// Test if min reserve check works swapping first token to the second token  
fn test_min_reserve_swap_exact_in(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    initial_reserves: Vec<u128>,
    swap_amount_in: u128,
    expected_result: Result<(u128, u128), StablePoolError>,
) {
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        session,
        token_decimals,
        initial_reserves.clone().iter().map(|a| a * 10).collect(), // increase tokens supply so there's enough tokens for swapping
        1000,
        25,
        6,
        BOB,
    );
    _ = stable_swap::add_liquidity(session, stable_swap, BOB, 1, initial_reserves, bob());

    let res = stable_swap::swap_exact_in(
        session,
        stable_swap,
        BOB,
        tokens[0],
        tokens[1], // min withdraw amounts
        swap_amount_in,
        1,
        bob(),
    );
    assert_eq!(expected_result, res, "Unexpected result");
}

/// Test if min reserve check works swapping first token to the second token  
fn test_min_reserve_swap_received(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    initial_reserves: Vec<u128>,
    swap_amount_in: u128,
    expected_result: Result<(u128, u128), StablePoolError>,
) {
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        session,
        token_decimals,
        initial_reserves.clone().iter().map(|a| a * 10).collect(), // increase tokens supply so there's enough tokens for swapping
        1000,
        25,
        6,
        BOB,
    );
    _ = stable_swap::add_liquidity(session, stable_swap, BOB, 1, initial_reserves, bob());

    _ = psp22_utils::transfer(session, tokens[0], stable_swap, swap_amount_in, BOB).expect("Transfer failed");

    let res = stable_swap::swap_received(
        session,
        stable_swap,
        BOB,
        tokens[0],
        tokens[1], // min withdraw amounts
        1,
        bob(),
    );
    assert_eq!(expected_result, res, "Unexpected result");
}

/// Test if min reserve check works swapping first token to the second token  
fn test_min_reserve_swap_exact_out(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    initial_reserves: Vec<u128>,
    swap_amount_out: u128,
    expected_result: Result<(u128, u128), StablePoolError>,
) {
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        session,
        token_decimals,
        initial_reserves.clone().iter().map(|a| a * 10).collect(), // increase tokens supply so there's enough tokens for swapping
        1000,
        25,
        6,
        BOB,
    );
    _ = stable_swap::add_liquidity(session, stable_swap, BOB, 1, initial_reserves, bob());

    let res = stable_swap::swap_exact_out(
        session,
        stable_swap,
        BOB,
        tokens[0],
        tokens[1], // min withdraw amounts
        swap_amount_out,
        u128::MAX,
        bob(),
    );
    assert_eq!(expected_result, res, "Unexpected result");
}

#[drink::test]
fn test_01(session: &mut Session) {
    // one 1 usdt, 1 usdc
    test_min_initial_reserve(
        &mut session,
        vec![USDT_DEC, USDC_DEC],
        vec![1_000_000u128, 1_000_000u128],
        Ok((2000000000000000000, 0)),
    );
}

#[drink::test]
fn test_02(session: &mut Session) {
    // one 0.1 usdt, 0.1 usdc
    test_min_initial_reserve(
        &mut session,
        vec![USDT_DEC, USDC_DEC],
        vec![100_000u128, 100_000u128],
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_03(session: &mut Session) {
    // one 1 usdt, 0.1 usdc
    test_min_initial_reserve(
        &mut session,
        vec![USDT_DEC, USDC_DEC],
        vec![1_000_000u128, 100_000u128],
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_04(session: &mut Session) {
    // one 1 usdt, 1 dai
    test_min_initial_reserve(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![1_000_000u128, 1_000_000_000_000_000_000u128],
        Ok((2000000000000000000, 0)),
    );
}

#[drink::test]
fn test_05(session: &mut Session) {
    // one 1 usdt, 0.1 dai
    test_min_initial_reserve(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![1_000_000u128, 100_000_000_000_000_000u128],
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_06(session: &mut Session) {
    // reserves one 1 usdt, 1 dai
    // withdraw 1 usdt, 1 dai
    test_min_reserve_withdraw_liquidity(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![1_000_000u128, 1_000_000_000_000_000_000u128],
        vec![1_000_000u128, 1_000_000_000_000_000_000u128],
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_07(session: &mut Session) {
    // reserves one 1 usdt, 1 dai
    // withdraw 0.1 usdt, 0.1 dai
    test_min_reserve_withdraw_liquidity(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![1_000_000u128, 1_000_000_000_000_000_000u128],
        vec![100_000u128, 100_000_000_000_000_000u128],
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_08(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // withdraw 1 usdt, 1 dai
    test_min_reserve_withdraw_liquidity(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        vec![1_000_000u128, 1_000_000_000_000_000_000u128],
        Ok((2000000000000000000, 0)),
    );
}

#[drink::test]
fn test_09(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // withdraw 1.1 usdt, 1.1 dai
    test_min_reserve_withdraw_liquidity(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        vec![1_100_000u128, 1_100_000_000_000_000_000u128],
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_10(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // withdraw 1.5 usdt, 0.5 dai
    test_min_reserve_withdraw_liquidity(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        vec![1_500_000u128, 500_000_000_000_000_000u128],
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_11(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // withdraw 0.5 usdt, 0.5 dai
    test_min_reserve_withdraw_liquidity(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        vec![500_000u128, 500_000_000_000_000_000u128],
        Ok((1000000000000000000, 0)),
    );
}

#[drink::test]
fn test_12(session: &mut Session) {
    // reserves one 1 usdt, 1 dai
    // withdraw 1 usdt, 1 dai
    test_min_reserve_withdraw_liquidity_by_shares(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![1_000_000u128, 1_000_000_000_000_000_000u128],
        2000000000000000000, // 100%
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_13(session: &mut Session) {
    // reserves one 1 usdt, 1 dai
    // withdraw 0.1 usdt, 0.1 dai
    test_min_reserve_withdraw_liquidity_by_shares(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![1_000_000u128, 1_000_000_000_000_000_000u128],
        2000000000000000000 / 10, // 10%
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_14(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // withdraw 1 usdt, 1 dai
    test_min_reserve_withdraw_liquidity_by_shares(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        2000000000000000000, // 50%
        Ok(vec![1_000_000u128, 1_000_000_000_000_000_000u128]),
    );
}

#[drink::test]
fn test_15(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // withdraw 1.1 usdt, 1.1 dai
    test_min_reserve_withdraw_liquidity_by_shares(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        4000000000000000000 * 51 / 100, // 51 %
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_16(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // withdraw 1.5 usdt, 0.5 dai
    test_min_reserve_withdraw_liquidity_by_shares(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        4000000000000000000 * 3 / 4, // 75%
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_17(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // withdraw 0.5 usdt, 0.5 dai
    test_min_reserve_withdraw_liquidity_by_shares(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        4000000000000000000 / 4, // 25%
        Ok(vec![500_000u128, 500_000_000_000_000_000u128]),
    );
}

#[drink::test]
fn test_18(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // swap 1 usdt, expect less than 1 dai (min reserve holds)
    test_min_reserve_swap_exact_in(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        1_000_000u128,
        Ok((997167942595869932, 2499167775929498)),
    );
}

#[drink::test]
fn test_19(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // swap 2 usdt, expect more than 1 dai (min reserve check trigged)
    test_min_reserve_swap_exact_in(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        2_000_000u128,
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_20(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // swap 1 usdt, expect less than 1 dai (min reserve holds)
    test_min_reserve_swap_received(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        1_000_000u128,
        Ok((997167942595869932, 2499167775929498)),
    );
}

#[drink::test]
fn test_21(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // swap 2 usdt, expect more than 1 dai (min reserve check trigged)
    test_min_reserve_swap_received(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        2_000_000u128,
        Err(StablePoolError::MinReserve()),
    );
}

#[drink::test]
fn test_22(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // expect less than 1 dai (min reserve holds)
    test_min_reserve_swap_exact_out(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        997167942595869932u128,
        Ok((1_000_000u128, 2499167775929498)),
    );
}

#[drink::test]
fn test_23(session: &mut Session) {
    // reserves one 2 usdt, 2 dai
    // expect more than 1 dai (min reserve check trigged)
    test_min_reserve_swap_exact_out(
        &mut session,
        vec![USDT_DEC, DAI_DEC],
        vec![2_000_000u128, 2_000_000_000_000_000_000u128],
        1_100_000_000_000_000_000u128,
        Err(StablePoolError::MinReserve()),
    );
}
