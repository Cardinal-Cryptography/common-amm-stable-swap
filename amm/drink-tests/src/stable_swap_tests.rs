use crate::stable_pool_contract::{self, StablePoolError};
use crate::utils::*;

use drink::frame_support::sp_runtime::traits::IntegerSquareRoot;
use drink::frame_support::sp_runtime::traits::Scale;
use drink::{self, runtime::MinimalRuntime, session::Session};
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ToAccountId};

const FEE_BPS_DENOM: u128 = 10_000;

fn setup_stable_swap(
    session: &mut Session<MinimalRuntime>,
    token_decimals: [u8; 2],
    token_supply: [u128; 2],
    amp_coef: u128,
    fee_bps: u16,
    protocol_fee_bps: u16,
) -> (AccountId, AccountId, AccountId) {
    // (stable_pool, token_0, token_1)
    let _ = session.set_actor(BOB);
    upload_all(session);
    session
        .upload_code(stable_pool_contract::upload())
        .expect("Upload stable_pair_contract code");

    // instantiate tokens
    let token_0 = psp22_utils::setup_with_amounts(
        session,
        "token_0".to_string(),
        token_decimals[0],
        token_supply[0],
        BOB,
    );
    let token_1 = psp22_utils::setup_with_amounts(
        session,
        "token_1".to_string(),
        token_decimals[1],
        token_supply[1],
        BOB,
    );

    // instantiate stable_swap
    let instance = stable_pool_contract::Instance::new_stable(
        vec![token_0.into(), token_1.into()],
        token_decimals.to_vec(),
        amp_coef,
        bob(),
        fee_bps,
        protocol_fee_bps,
        Some(charlie()), // fee receiver
    );

    let stable_swap: stable_pool_contract::Instance = session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into();

    // setup max allowance for stable swap contract on both tokens
    for token in [token_0, token_1] {
        psp22_utils::increase_allowance(session, token.into(), stable_swap.into(), u128::MAX, BOB)
            .unwrap();
    }

    (stable_swap.into(), token_0.into(), token_1.into())
}

/// Tests swap of token at index 0 to token at index 1.
fn setup_test_swap_exact_in(
    session: &mut Session<MinimalRuntime>,
    token_decimals: [u8; 2],
    initial_reserves: [u128; 2],
    amp_coef: u128,
    fee_bps: u16,
    protocol_fee_bps: u16,
    swap_amount_in: u128,
    expected_swap_amount_out_total_result: Result<u128, StablePoolError>,
) {
    let initial_supply = [initial_reserves[0] + swap_amount_in, initial_reserves[1]];
    let (stable_swap, token_0, token_1) = setup_stable_swap(
        session,
        token_decimals,
        initial_supply,
        amp_coef,
        fee_bps,
        protocol_fee_bps,
    );
    _ = stable_swap::add_liquidity(
        session,
        stable_swap,
        BOB,
        1,
        initial_reserves.to_vec(),
        bob(),
    );

    let swap_result = stable_swap::swap_exact_in(
        session,
        stable_swap.into(),
        BOB,
        token_0,        // in
        token_1,        // out
        swap_amount_in, // amount_in
        0,              // min_token_out
        bob(),
    )
    .result
    .unwrap();

    if expected_swap_amount_out_total_result.is_err() {
        match swap_result {
            Err(ref err) => {
                let wrapped_error: Result<u128, StablePoolError> = Err(err.clone());
                assert_eq!(expected_swap_amount_out_total_result, wrapped_error);
                return;
            }
            Ok(val) => panic!("Should return an error. Return val: {val:?}"),
        }
    }

    let (amount_out, fee) = swap_result.unwrap();
    let expected_swap_amount_out_total = expected_swap_amount_out_total_result.unwrap();
    let expected_fee = expected_swap_amount_out_total * fee_bps as u128 / FEE_BPS_DENOM;
    let expected_swap_amount_out = expected_swap_amount_out_total - expected_fee;
    let expected_protocol_fee_part = expected_fee * protocol_fee_bps as u128 / FEE_BPS_DENOM;

    // check returned amount swapped and fee
    assert_eq!(expected_swap_amount_out, amount_out, "Amount out mismatch");
    assert_eq!(expected_fee, fee, "Fee mismatch");

    // check if reserves are equal the actual balances
    let reserves = stable_swap::reserves(session, stable_swap.into());
    let balance_0 = psp22_utils::balance_of(session, token_0.into(), stable_swap.into());
    let balance_1 = psp22_utils::balance_of(session, token_1.into(), stable_swap.into());
    assert_eq!(
        reserves,
        vec![balance_0, balance_1],
        "Balances - reserves mismatch"
    );

    // check bobs balances
    let balance_0 = psp22_utils::balance_of(session, token_0.into(), bob());
    let balance_1 = psp22_utils::balance_of(session, token_1.into(), bob());
    assert_eq!(
        [0, expected_swap_amount_out],
        [balance_0, balance_1],
        "Incorrect Bob's balances"
    );

    // check protocol fee
    let protocol_fee_lp = psp22_utils::balance_of(session, stable_swap.into(), charlie());
    let (total_lp_required, lp_fee_part) = stable_swap::remove_liquidity_by_amounts(
        session,
        stable_swap.into(),
        BOB,
        protocol_fee_lp * 2,
        [0, expected_protocol_fee_part].to_vec(),
        bob(),
    )
    .result
    .unwrap()
    .unwrap();
    assert_eq!(
        total_lp_required - lp_fee_part,
        protocol_fee_lp,
        "Incorrect protocol fee"
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L744
#[drink::test]
fn test_stable_swap_exact_in_01(mut session: Session) {
    setup_test_swap_exact_in(
        &mut session,
        [6, 6],                       // decimals
        [100000000000, 100000000000], // initial reserves
        1000,                         // A
        6,                            // fee BPS
        2000,                         // protocol fee BPS
        10000000000,                  // swap_amount_in
        Ok(9999495232),               // expected out (with fee)
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L763
#[drink::test]
fn test_stable_swap_exact_in_02(mut session: Session) {
    setup_test_swap_exact_in(
        &mut session,
        [12, 18],
        [100000000000000000, 100000000000000000000000],
        1000,
        6,
        2000,
        10000000000000000,
        Ok(9999495232752197989995),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L782
#[drink::test]
fn test_stable_swap_exact_in_03(mut session: Session) {
    setup_test_swap_exact_in(
        &mut session,
        [6, 6],
        [100000000000, 100000000000],
        1000,
        6,
        2000,
        0,
        Err(StablePoolError::InsufficientInputAmount()),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L801
#[drink::test]
fn test_stable_swap_exact_in_04(mut session: Session) {
    setup_test_swap_exact_in(
        &mut session,
        [12, 18],
        [100000000000000000, 100000000000000000000000],
        1000,
        6,
        2000,
        0,
        Err(StablePoolError::InsufficientInputAmount()),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L820
#[drink::test]
fn test_stable_swap_exact_in_05(mut session: Session) {
    setup_test_swap_exact_in(
        &mut session,
        [6, 6],
        [100000000000, 100000000000],
        1000,
        6,
        2000,
        1,
        Ok(0),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L839
#[drink::test]
fn test_stable_swap_exact_in_06(mut session: Session) {
    setup_test_swap_exact_in(
        &mut session,
        [18, 12],
        [100000000000000000000000, 100000000000000000],
        1000,
        6,
        2000,
        1000000,
        Ok(0),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L858
#[drink::test]
fn test_stable_swap_exact_in_07(mut session: Session) {
    setup_test_swap_exact_in(
        &mut session,
        [6, 6],
        [100000000000, 100000000000],
        1000,
        6,
        2000,
        100000000000,
        Ok(98443663539),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L877
#[drink::test]
fn test_stable_swap_exact_in_08(mut session: Session) {
    setup_test_swap_exact_in(
        &mut session,
        [12, 18],
        [100000000000000000, 100000000000000000000000],
        1000,
        6,
        2000,
        100000000000000000,
        Ok(98443663539913153080656),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L896
#[drink::test]
fn test_stable_swap_exact_in_09(mut session: Session) {
    setup_test_swap_exact_in(
        &mut session,
        [6, 6],
        [100000000000, 100000000000],
        1000,
        6,
        2000,
        99999000000 + 1, // +1 because of accounting for fee rounding
        Ok(98443167413),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L915
#[drink::test]
fn test_stable_swap_exact_in_10(mut session: Session) {
    setup_test_swap_exact_in(
        &mut session,
        [12, 18],
        [100000000000000000, 100000000000000000000000],
        1000,
        6,
        2000,
        99999000000000000,
        Ok(98443167413204135506296),
    );
}
