use drink::{self, session::Session};
use stable_pool_contract::MathError;

use super::*;

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/tests/test_stable_pool.rs#L123
#[drink::test]
fn test_01(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);
    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        25,
        2000,
        BOB,
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, stable_swap);

    transfer_and_increase_allowance(
        &mut session,
        stable_swap,
        tokens.clone(),
        CHARLIE,
        vec![500 * ONE_DAI, 500 * ONE_USDT, 500 * ONE_USDC],
        BOB,
    );

    // add more liquidity with balanced tokens (charlie)
    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        CHARLIE,
        1,
        vec![500 * ONE_DAI, 500 * ONE_USDT, 500 * ONE_USDC],
        charlie(),
    )
    .expect("Should successfully add liquidity");

    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares + 1500 * ONE_LPT)
    );

    let last_total_shares = last_total_shares + 1500 * ONE_LPT;

    // remove by shares (charlie)
    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        CHARLIE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 1 * ONE_USDT, 1 * ONE_USDC],
        charlie(),
    )
    .expect("Should successfully remove liquidity");

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1200 * ONE_LPT
    );
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, charlie()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances,
        vec![100 * ONE_DAI, 100 * ONE_USDT, 100 * ONE_USDC],
        "Incorrect Users tokens balances"
    );
    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares - 300 * ONE_LPT)
    );
    let last_total_shares = last_total_shares - 300 * ONE_LPT;

    transfer_and_increase_allowance(
        &mut session,
        stable_swap,
        tokens.clone(),
        DAVE,
        vec![100 * ONE_DAI, 200 * ONE_USDT, 400 * ONE_USDC],
        BOB,
    );

    // add more liquidity with imbalanced tokens (dave)
    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        DAVE,
        1,
        vec![100 * ONE_DAI, 200 * ONE_USDT, 400 * ONE_USDC],
        dave(),
    )
    .expect("Should successfully add liquidity");
    // "Mint 699699997426210330025 shares for user2, fee is 299999998348895348 shares",
    // "Exchange swap got 59999999669779069 shares",

    assert_eq!(
        stable_swap::reserves(&mut session, stable_swap),
        vec![100500 * ONE_DAI, 100600 * ONE_USDT, 100800 * ONE_USDC],
        "Incorrect reserves"
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        301200 * ONE_LPT + 699699997426210330025 + 59999999669779069,
        "Incorrect total shares"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699699997426210330025,
        "Incorrect Users share"
    );

    let (current_share_price, current_total_shares) =
        share_price_and_total_shares(&mut session, stable_swap);
    assert!(
        current_share_price > last_share_price,
        "Incorrect share price"
    );
    let last_share_price = current_share_price;

    assert_eq!(
        current_total_shares,
        last_total_shares + 699699997426210330025 + 59999999669779069
    );
    let last_total_shares = current_total_shares;

    // remove by tokens (charlie)
    _ = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        CHARLIE,
        550 * ONE_LPT,
        vec![1 * ONE_DAI, 500 * ONE_USDT, 1 * ONE_USDC],
        charlie(),
    )
    .expect("Should successfully remove liquidity. Err: {err:?}");
    // "LP charlie removed 502598511257512352631 shares by given tokens, and fee is 598899301432400519 shares",
    // "Exchange swap got 119779860286480103 shares",
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1200 * ONE_LPT - 502598511257512352631,
        "Incorrect users share"
    );

    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, charlie()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances,
        vec![101 * ONE_DAI, 600 * ONE_USDT, 101 * ONE_USDC],
        "Incorrect Users tokens balances"
    );

    assert_eq!(
        stable_swap::reserves(&mut session, stable_swap),
        vec![100499 * ONE_DAI, 100100 * ONE_USDT, 100799 * ONE_USDC],
        "Incorrect reserves"
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        last_total_shares - 502598511257512352631 + 119779860286480103,
        "Incorrect total shares"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1200 * ONE_LPT - 502598511257512352631,
        "Incorrect users share"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699699997426210330025,
        "Incorrect users share"
    );
    let (current_share_price, _) = share_price_and_total_shares(&mut session, stable_swap);
    assert!(
        current_share_price > last_share_price,
        "Incorrect share price"
    );
    let last_share_price = current_share_price;
    let last_total_shares = last_total_shares - 502598511257512352631 + 119779860286480103;

    // transfer some LPT to from charlie to dave
    _ = psp22_utils::transfer(&mut session, stable_swap, dave(), 100 * ONE_LPT, CHARLIE);

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1100 * ONE_LPT - 502598511257512352631,
        "Incorrect user balance"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699699997426210330025 + 100 * ONE_LPT,
        "Incorrect user balance"
    );

    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares),
        "Incorrect share price and/or total shares"
    );

    // dave remove by shares trigger slippage
    let res = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        DAVE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 298 * ONE_USDT, 1 * ONE_USDC],
        dave(),
    )
    .expect_err("Should return an error");

    assert_eq!(
        res,
        StablePoolError::InsufficientOutputAmount(),
        "Should return correct error"
    );

    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares),
        "Incorrect share price and/or total shares"
    );

    // dave remove by tokens trigger slippage
    let res = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        DAVE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 298 * ONE_USDT, 1 * ONE_USDC],
        dave(),
    )
    .expect_err("Should return an error");

    assert_eq!(
        res,
        StablePoolError::InsufficientLiquidityBurned(),
        "Should return correct error"
    );

    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares),
        "Incorrect share price and/or total shares"
    );

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1100 * ONE_LPT - 502598511257512352631,
        "Incorrect user balance"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699699997426210330025 + 100 * ONE_LPT,
        "Incorrect user balance"
    );

    // dave remove by share
    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        DAVE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 1 * ONE_USDT, 1 * ONE_USDC],
        dave(),
    )
    .expect("Should successfully remove liquidity");
    // "LP dave removed 498596320225563082252 shares by given tokens, and fee is 597500435701476809 shares",
    // "Exchange swap got 119500087140295361 shares",

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1100 * ONE_LPT - 502598511257512352631,
        "Incorrect user balance"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699699997426210330025 - 200 * ONE_LPT,
        "Incorrect user balance"
    );

    let (current_share_price, current_total_shares) =
        share_price_and_total_shares(&mut session, stable_swap);
    assert_eq!(
        current_share_price, last_share_price,
        "Incorrect share price"
    );
    assert_eq!(
        current_total_shares,
        last_total_shares - 300 * ONE_LPT,
        "Incorrect total shares"
    );
    let last_total_shares = last_total_shares - 300 * ONE_LPT;

    _ = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        DAVE,
        499 * ONE_LPT,
        vec![498 * ONE_DAI, 0 * ONE_USDT, 0 * ONE_USDC],
        dave(),
    )
    .expect("Should successfully remove liquidity");
    // "LP user2 removed 498596320225563082252 shares by given tokens, and fee is 597500435701476809 shares",
    // "Exchange swap got 119500087140295361 shares, No referral fee, from remove_liquidity_by_tokens",
    // -- DIFF --
    // "LP dave removed 498596320224035614380 shares by given tokens, and fee is 597500435700561479 shares",
    // "Exchange swap got 119500087140112295 shares, No referral fee (not implemented)",
    //
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1100 * ONE_LPT - 502598511257512352631,
        "Incorrect user balance"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699699997426210330025 - 200 * ONE_LPT - 498596320224035614380,
        "Incorrect user balance"
    );

    let last_total_shares = last_total_shares - 498596320224035614380 + 119500087140112295;
    let (current_share_price, current_total_shares) =
        share_price_and_total_shares(&mut session, stable_swap);
    assert!(
        current_share_price > last_share_price,
        "Incorrect share price"
    );
    assert_eq!(
        current_total_shares, last_total_shares,
        "Incorrect total shares"
    );

    transfer_and_increase_allowance(
        &mut session,
        stable_swap,
        tokens.clone(),
        EVA,
        vec![
            100_000_000_000 * ONE_DAI,
            100_000_000_000 * ONE_USDT,
            100_000_000_000 * ONE_USDC,
        ],
        BOB,
    );

    stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        EVA,
        1,
        vec![
            100_000_000_000 * ONE_DAI,
            100_000_000_000 * ONE_USDT,
            100_000_000_000 * ONE_USDC,
        ],
        eva(),
    )
    .expect("Should successfully add liquidity");
    // "Mint 299997911758886758506069372942 shares for user3, fee is 895808190595468286848457 shares",
    // "Exchange swap got 179161638119093657369691 shares, No referral fee, from add_liquidity",
    // -- DIFF --
    // "Mint 299997911757966485300035937427 shares for eva, fee is 895808191250701043141970 shares",
    // "Exchange swap got 179161638250140208628394 shares, No referral fee (not implemented)",
    //
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, eva()),
        299997911757966485300035937427,
        "Incorrect user balance"
    );

    let last_total_shares =
        last_total_shares + 299997911757966485300035937427 + 179161638250140208628394;
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        last_total_shares,
        "Incorrect total shares"
    );
}

/// Test withdrawing all liquidity with all shares
#[drink::test]
fn test_02(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        25,
        2000,
        BOB,
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    // remove by shares
    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        BOB,
        300000 * ONE_LPT,
        vec![1 * ONE_DAI, 1 * ONE_USDT, 1 * ONE_USDC],
        bob(),
    )
    .expect("Should successfully remove liquidity");

    assert_eq!(psp22_utils::balance_of(&mut session, stable_swap, bob()), 0);
    assert_eq!(psp22_utils::total_supply(&mut session, stable_swap), 0);
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(balances, initial_supply, "Incorrect Users tokens balances");
}

/// Test withdrawing all liquidity by amounts
#[drink::test]
fn test_03(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        25,
        2000,
        BOB,
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    _ = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        300000 * ONE_LPT,
        initial_reserves,
        bob(),
    )
    .expect("Should successfully remove liquidity");

    assert_eq!(psp22_utils::balance_of(&mut session, stable_swap, bob()), 0);
    assert_eq!(psp22_utils::total_supply(&mut session, stable_swap), 0);
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(balances, initial_supply, "Incorrect Users tokens balances");
}

/// Test withdrawing all liquidity with shares - 1
#[drink::test]
fn test_04(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let initial_supply_sub_reserves = initial_supply
        .iter()
        .zip(initial_reserves.iter())
        .map(|(supply, reserve)| supply - reserve)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        25,
        2000,
        BOB,
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    let err = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        BOB,
        300000 * ONE_LPT - 1,
        initial_reserves.clone(),
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::InsufficientOutputAmount(),
        "Should return appropriate error"
    );

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        300000 * ONE_LPT - 1,
        initial_reserves,
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::InsufficientLiquidityBurned(),
        "Should return appropriate error"
    );

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, bob()),
        300000 * ONE_LPT
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        300000 * ONE_LPT
    );
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances, initial_supply_sub_reserves,
        "Incorrect Users tokens balances"
    );
}

/// Test withdrawing single token whole reserve
#[drink::test]
fn test_05(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let initial_supply_sub_reserves = initial_supply
        .iter()
        .zip(initial_reserves.iter())
        .map(|(supply, reserve)| supply - reserve)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        25,
        2000,
        BOB,
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        300000 * ONE_LPT,
        vec![initial_reserves[0], 0, 0],
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");

    assert_eq!(
        err,
        StablePoolError::MathError(MathError::DivByZero(1)),
        "Should return appropriate error"
    );

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, bob()),
        300000 * ONE_LPT
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        300000 * ONE_LPT
    );
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances, initial_supply_sub_reserves,
        "Incorrect Users tokens balances"
    );
}

/// Test withdrawing all liquidity with shares - 1 (with different initial reserves)
#[drink::test]
fn test_06(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![543257 * ONE_DAI, 123123 * ONE_USDT, 32178139 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let initial_supply_sub_reserves = initial_supply
        .iter()
        .zip(initial_reserves.iter())
        .map(|(supply, reserve)| supply - reserve)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        25,
        2000,
        BOB,
    );

    let (shares, _) = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    let err = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        BOB,
        shares - 1,
        initial_reserves.clone(),
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::InsufficientOutputAmount(),
        "Should return appropriate error"
    );

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        shares - 1,
        initial_reserves,
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::InsufficientLiquidityBurned(),
        "Should return appropriate error"
    );

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, bob()),
        shares
    );
    assert_eq!(psp22_utils::total_supply(&mut session, stable_swap), shares);
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances, initial_supply_sub_reserves,
        "Incorrect Users tokens balances"
    );
}

/// Test withdrawing single token whole reserve (with different initial reserves)
#[drink::test]
fn test_07(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![543257 * ONE_DAI, 123123 * ONE_USDT, 32178139 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let initial_supply_sub_reserves = initial_supply
        .iter()
        .zip(initial_reserves.iter())
        .map(|(supply, reserve)| supply - reserve)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        25,
        2000,
        BOB,
    );

    let (shares, _) = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        shares,
        vec![initial_reserves[0], 0, 0],
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::MathError(MathError::DivByZero(1)),
        "Should return appropriate error"
    );

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        shares,
        vec![0, initial_reserves[1], 0],
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::MathError(MathError::DivByZero(1)),
        "Should return appropriate error"
    );

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        shares,
        vec![0, 0, initial_reserves[2]],
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::MathError(MathError::DivByZero(1)),
        "Should return appropriate error"
    );

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, bob()),
        shares
    );
    assert_eq!(psp22_utils::total_supply(&mut session, stable_swap), shares);
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances, initial_supply_sub_reserves,
        "Incorrect Users tokens balances"
    );
}
