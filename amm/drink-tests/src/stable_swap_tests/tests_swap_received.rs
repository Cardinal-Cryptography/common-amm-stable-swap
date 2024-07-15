use drink::{self, runtime::MinimalRuntime, session::Session};

use super::*;

/// Tests swap of token at index 0 to token at index 1.
fn test_swap_received(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    initial_reserves: Vec<u128>,
    amp_coef: u128,
    trad_fee: u32,
    protocol_fee: u32,
    swap_amount_in: u128,
    expected_swap_amount_out_total_result: Result<u128, StablePoolError>,
) {
    let initial_supply = vec![initial_reserves[0] + swap_amount_in, initial_reserves[1]];
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        session,
        token_decimals,
        initial_supply,
        amp_coef,
        trad_fee,
        protocol_fee,
        BOB,
        vec![],
    );
    _ = stable_swap::add_liquidity(
        session,
        stable_swap,
        BOB,
        1,
        initial_reserves.to_vec(),
        bob(),
    );

    let _ = psp22_utils::transfer(session, tokens[0], stable_swap, swap_amount_in, BOB);

    let swap_result = stable_swap::swap_received(
        session,
        stable_swap,
        BOB,
        tokens[0], // in
        tokens[1], // out
        0,         // min_token_out
        bob(),
    );

    if expected_swap_amount_out_total_result.is_err() {
        match swap_result {
            Err(err) => {
                let wrapped_error: Result<u128, StablePoolError> = Err(err);
                assert_eq!(expected_swap_amount_out_total_result, wrapped_error);
                return;
            }
            Ok(val) => panic!("Should return an error. Return val: {val:?}"),
        }
    }

    let (amount_out, fee) = swap_result.unwrap();
    let expected_swap_amount_out_total = expected_swap_amount_out_total_result.unwrap();
    let expected_fee = expected_swap_amount_out_total * trad_fee as u128 / FEE_DENOM;
    let expected_swap_amount_out = expected_swap_amount_out_total - expected_fee;
    let expected_protocol_fee_part = expected_fee * protocol_fee as u128 / FEE_DENOM;

    // check returned amount swapped and fee
    assert_eq!(expected_swap_amount_out, amount_out, "Amount out mismatch");
    assert_eq!(expected_fee, fee, "Fee mismatch");

    // check if reserves were updated properly
    let reserves = stable_swap::reserves(session, stable_swap);
    assert_eq!(
        reserves,
        [
            initial_reserves[0] + swap_amount_in,
            initial_reserves[1] - expected_swap_amount_out
        ],
        "Reserves not updated properly"
    );

    // check if reserves are equal the actual balances
    let balance_0 = psp22_utils::balance_of(session, tokens[0], stable_swap);
    let balance_1 = psp22_utils::balance_of(session, tokens[1], stable_swap);
    assert_eq!(
        reserves,
        vec![balance_0, balance_1],
        "Balances - reserves mismatch"
    );

    // check bobs balances
    let balance_0 = psp22_utils::balance_of(session, tokens[0], bob());
    let balance_1 = psp22_utils::balance_of(session, tokens[1], bob());
    assert_eq!(
        [0, expected_swap_amount_out],
        [balance_0, balance_1],
        "Incorrect Bob's balances"
    );

    // check protocol fee
    let protocol_fee_lp = psp22_utils::balance_of(session, stable_swap.into(), fee_receiver());
    let (total_lp_required, lp_fee_part) = stable_swap::remove_liquidity_by_amounts(
        session,
        stable_swap.into(),
        BOB,
        protocol_fee_lp * 2,
        [0, expected_protocol_fee_part].to_vec(),
        bob(),
    )
    .expect("Should remove lp");
    assert_eq!(
        total_lp_required - lp_fee_part,
        protocol_fee_lp,
        "Incorrect protocol fee"
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L744
#[drink::test]
fn test_01(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![6, 6],                       // decimals
        vec![100000000000, 100000000000], // initial reserves
        1000,                             // A
        600_000,                          // trade fee in 1e9 precision
        200_000_000,                      // protocol fee in 1e9 precision
        10000000000,                      // swap_amount_in
        Ok(9999495232),                   // expected out (with fee)
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L763
#[drink::test]
fn test_02(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![12, 18],
        vec![100000000000000000, 100000000000000000000000],
        1000,
        600_000,
        200_000_000,
        10000000000000000,
        Ok(9999495232752197989995),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L782
#[drink::test]
fn test_03(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![6, 6],
        vec![100000000000, 100000000000],
        1000,
        600_000,
        200_000_000,
        0,
        Err(StablePoolError::InsufficientInputAmount()),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L801
#[drink::test]
fn test_04(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![12, 18],
        vec![100000000000000000, 100000000000000000000000],
        1000,
        600_000,
        200_000_000,
        0,
        Err(StablePoolError::InsufficientInputAmount()),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L820
#[drink::test]
fn test_05(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![6, 6],
        vec![100000000000, 100000000000],
        1000,
        600_000,
        200_000_000,
        1,
        Ok(0),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L839
// Test that swapping 0.000000000001000000 gives 0.000000000000 (token precision cut)
#[drink::test]
fn test_06_a(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![18, 12],
        vec![100000000000000000000000, 100000000000000000],
        1000,
        600_000,
        200_000_000,
        1000000,
        Ok(0),
    );
}

// Test that swapping (with disabled fees) 0.000000000001000000 gives 0.000000000000
#[drink::test]
fn test_06_b(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![18, 12],
        vec![100000000000000000000000, 100000000000000000],
        1000,
        0,
        0,
        1000000,
        Ok(0),
    );
}

/// Test that swapping (with disabled fees) 0.000000000001000001 gives 0.000000000001
#[drink::test]
fn test_06_c(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![18, 12],
        vec![100000000000000000000000, 100000000000000000],
        1000,
        0,
        0,
        1000001,
        Ok(1),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L858
#[drink::test]
fn test_07(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![6, 6],
        vec![100000000000, 100000000000],
        1000,
        600_000,
        200_000_000,
        100000000000,
        Ok(98443663539),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L877
#[drink::test]
fn test_08(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![12, 18],
        vec![100000000000000000, 100000000000000000000000],
        1000,
        600_000,
        200_000_000,
        100000000000000000,
        Ok(98443663539913153080656),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L896
#[drink::test]
fn test_09(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![6, 6],
        vec![100000000000, 100000000000],
        1000,
        600_000,
        200_000_000,
        99999000000 + 1, // +1 because of accounting for fee rounding
        Ok(98443167413),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L915
#[drink::test]
fn test_10(mut session: Session) {
    test_swap_received(
        &mut session,
        vec![12, 18],
        vec![100000000000000000, 100000000000000000000000],
        1000,
        600_000,
        200_000_000,
        99999000000000000,
        Ok(98443167413204135506296),
    );
}
