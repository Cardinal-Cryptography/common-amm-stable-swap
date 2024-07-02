use drink::{self, session::Session};

const ONE_LPT: u128 = 1000000000000000000;
const ONE_DAI: u128 = 1000000000000000000;
const ONE_USDT: u128 = 1000000;
const ONE_USDC: u128 = 1000000;

use super::*;

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/tests/test_stable_pool.rs#L23
#[drink::test]
fn test_01(mut session: Session) {
    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_reserves.iter().map(|amount| amount * 10).collect(),
        10_000,
        25,
        2000,
        BOB,
    );

    handle_ink_error(stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    ))
    .unwrap_or_else(|err| panic!("Should successfully add liquidity. Err: {err:?}"));

    assert_eq!(
        stable_swap::tokens(&mut session, stable_swap),
        tokens,
        "Incorrect token accounts"
    );
    assert_eq!(
        stable_swap::reserves(&mut session, stable_swap),
        initial_reserves,
        "Incorrect reserves"
    );
    assert_eq!(
        stable_swap::amp_coef(&mut session, stable_swap),
        10_000,
        "Incorrect A"
    );
    assert_eq!(
        stable_swap::fees(&mut session, stable_swap),
        (25, 2000),
        "Incorrect fees"
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        300_000 * ONE_LPT,
        "Incorrect LP token supply"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, bob()),
        300_000 * ONE_LPT,
        "Incorrect Users LP token balance"
    );

    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect();
    assert_eq!(
        balances,
        initial_reserves
            .iter()
            .map(|amount| amount * 9)
            .collect::<Vec<u128>>(),
        "Incorrect Users tokens balances"
    );

    _ = handle_ink_error(stable_swap::swap_exact_in(
        &mut session,
        stable_swap,
        BOB,
        tokens[0], // DAI
        tokens[2], // USDC
        ONE_DAI,   // amount_in
        1,         // min_token_out
        charlie(),
    ))
    .unwrap_or_else(|err| panic!("Should successfully swap. Err: {err:?}"));

    _ = handle_ink_error(stable_swap::swap_exact_in(
        &mut session,
        stable_swap,
        BOB,
        tokens[0], // DAI
        tokens[1], // USDC
        ONE_DAI,   // amount_in
        1,         // min_token_out
        charlie(),
    ))
    .unwrap_or_else(|err| panic!("Should successfully swap. Err: {err:?}"));

    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, charlie()))
        .collect();
    assert_eq!(
        balances,
        vec![0, 997499, 997499],
        "Incorrect Users tokens balances"
    );

    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, stable_swap))
        .collect();
    assert_eq!(
        stable_swap::reserves(&mut session, stable_swap),
        balances,
        "Pool reserves and token balances mismatch"
    );

    assert_eq!(
        stable_swap::reserves(&mut session, stable_swap),
        vec![
            100002 * ONE_DAI,
            99999 * ONE_USDT + 2501, // -- DIFF -- 99999 * ONE_USDT + 2500
            99999 * ONE_USDC + 2501  // -- DIFF -- 99999 * ONE_USDC + 2500
        ],
        "Incorrect reserves"
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        300000 * ONE_LPT + 498999996725367 + 498999993395420, // -- DIFF -- 300000 * ONE_LPT + 499999996666583 + 499999993277742
        "Incorrect LP token supply"
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/tests/test_stable_pool.rs#L123
#[drink::test]
fn test_02(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);
    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_reserves
            .iter()
            .map(|amount| amount * 100_000_000_000)
            .collect(),
        10_000,
        25,
        2000,
        BOB,
    );

    handle_ink_error(stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    ))
    .unwrap_or_else(|err| panic!("Should successfully add liquidity. Err: {err:?}"));

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

    handle_ink_error(stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        CHARLIE,
        1,
        vec![500 * ONE_DAI, 500 * ONE_USDT, 500 * ONE_USDC],
        charlie(),
    ))
    .unwrap_or_else(|err| panic!("Should successfully add liquidity. Err: {err:?}"));

    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares + 1500 * ONE_LPT)
    );

    let last_total_shares = last_total_shares + 1500 * ONE_LPT;

    handle_ink_error(stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        CHARLIE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 1 * ONE_USDT, 1 * ONE_USDC],
        charlie(),
    ))
    .unwrap_or_else(|err| panic!("Should successfully remove liquidity. Err: {err:?}"));

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1200 * ONE_LPT
    );
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, charlie()))
        .collect();
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

    handle_ink_error(stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        DAVE,
        1,
        vec![100 * ONE_DAI, 200 * ONE_USDT, 400 * ONE_USDC],
        dave(),
    ))
    .unwrap_or_else(|err| panic!("Should successfully add liquidity. Err: {err:?}"));

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

    handle_ink_error(stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        CHARLIE,
        550 * ONE_LPT,
        vec![1 * ONE_DAI, 500 * ONE_USDT, 1 * ONE_USDC],
        charlie(),
    ))
    .unwrap_or_else(|err| panic!("Should successfully remove liquidity. Err: {err:?}"));

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1200 * ONE_LPT - 502598511257512352631,
        "Incorrect users share"
    );

    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, charlie()))
        .collect();
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
    let (current_share_price, current_total_shares) =
        share_price_and_total_shares(&mut session, stable_swap);
    assert!(
        current_share_price > last_share_price,
        "Incorrect share price"
    );
    let last_share_price = current_share_price;
    let last_total_shares = last_total_shares - 502598511257512352631 + 119779860286480103;

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

    let res = handle_ink_error(stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        DAVE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 298 * ONE_USDT, 1 * ONE_USDC],
        dave(),
    ));

    assert_eq!(
        res,
        Err(StablePoolError::InsufficientOutputAmount()),
        "Should return correct error"
    );

    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares),
        "Incorrect share price and/or total shares"
    );

    let res = handle_ink_error(stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        DAVE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 298 * ONE_USDT, 1 * ONE_USDC],
        dave(),
    ));

    assert_eq!(
        res,
        Err(StablePoolError::InsufficientLiquidityBurned()),
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

    handle_ink_error(stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        DAVE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 1 * ONE_USDT, 1 * ONE_USDC],
        dave(),
    ))
    .unwrap_or_else(|err| panic!("Should successfully remove liquidity. Err: {err:?}"));

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

    handle_ink_error(stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        DAVE,
        499 * ONE_LPT,
        vec![498 * ONE_DAI, 0 * ONE_USDT, 0 * ONE_USDC],
        dave(),
    ))
    .unwrap_or_else(|err| panic!("Should successfully remove liquidity. Err: {err:?}"));
    // "LP user2 removed 498596320225563082252 shares by given tokens, and fee is 597500435701476809 shares",
    // "Exchange swap got 119500087140295361 shares, No referral fee, from remove_liquidity_by_tokens",
    /* -- DIFF --
        "LP user2 removed 498596320224035614380 shares by given tokens, and fee is 597500435700561479 shares",
        "Exchange swap got 119500087140112295 shares, No referral fee (not implemented)",
    */
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
    let last_share_price = current_share_price;

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

    handle_ink_error(stable_swap::add_liquidity(
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
    ))
    .unwrap_or_else(|err| panic!("Should successfully add liquidity. Err: {err:?}"));
    // "Mint 299997911758886758506069372942 shares for user3, fee is 895808190595468286848457 shares",
    // "Exchange swap got 179161638119093657369691 shares, No referral fee, from add_liquidity",
    /* -- DIFF --
        "Mint 299997911757966485300035937427 shares for user3, fee is 895808191250701043141970 shares",
        "Exchange swap got 179161638250140208628394 shares, No referral fee, from add_liquidity",
    */
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
