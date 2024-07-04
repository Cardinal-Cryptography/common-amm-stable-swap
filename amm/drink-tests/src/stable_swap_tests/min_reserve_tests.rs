use drink::{runtime::MinimalRuntime, session::Session};
use super::*;

const USDT_DEC: u8 = 6;
const USDC_DEC: u8 = 6;
const DAI_DEC: u8 = 18;

fn test_min_initial_reserve(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    initial_reserves: Vec<u128>,
    expected_result: Result<(u128, u128), StablePoolError>,
) {
    let (stable_swap, _) =
        setup_stable_swap_with_tokens(session, token_decimals, initial_reserves.clone(), 1000, 25, 6, BOB);
    let res = stable_swap::add_liquidity(
        session,
        stable_swap,
        BOB,
        1,
        initial_reserves,
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
        vec![100_000u128, 100_000_000_000_000_000u128],
        Err(StablePoolError::MinReserve()),
    );
}
