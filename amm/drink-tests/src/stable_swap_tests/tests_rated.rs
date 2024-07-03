use crate::mock_sazero_rate_contract;
use crate::stable_pool_contract;
use crate::utils::*;

use super::*;

use drink::{self, runtime::MinimalRuntime, session::Session};
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ToAccountId};

const WAZERO_DEC: u8 = 12;
const SAZERO_DEC: u8 = 12;

const ONE_LPT: u128 = 10u128.pow(18);
const ONE_WAZERO: u128 = 10u128.pow(WAZERO_DEC as u32);
const ONE_SAZERO: u128 = 10u128.pow(SAZERO_DEC as u32);

const EXPIRE_TS: u64 = 24 * 3600 * 1000; // 24h

fn setup_rated_swap_with_tokens(
    session: &mut Session<MinimalRuntime>,
    caller: drink::AccountId32,
    initial_token_supply: u128,
    init_amp_coef: u128,
    rate_expiration_duration_ms: u64,
    trade_fee: u16,
    protocol_fee: u16,
) -> (AccountId, AccountId, AccountId, AccountId) {
    //upload and deploy rate mock
    session
        .upload_code(stable_pool_contract::upload())
        .expect("Upload stable_stable_pair_contract code");
    session
        .upload_code(mock_sazero_rate_contract::upload())
        .expect("Upload sazero_rate_mock_contract code");
    let _ = session.set_actor(caller.clone());

    let instance = mock_sazero_rate_contract::Instance::new();

    let wazero = psp22_utils::setup_with_amounts(
        session,
        "wAZERO".to_string(),
        WAZERO_DEC,
        initial_token_supply * ONE_WAZERO,
        caller.clone(),
    );
    let sazero = psp22_utils::setup_with_amounts(
        session,
        "SAZERO".to_string(),
        SAZERO_DEC,
        initial_token_supply * ONE_SAZERO,
        caller.clone(),
    );

    let rate_mock_address = session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into();
    let instance = stable_pool_contract::Instance::new_rated(
        vec![sazero.into(), wazero.into()],
        vec![SAZERO_DEC, WAZERO_DEC],
        vec![Some(rate_mock_address), None],
        rate_expiration_duration_ms,
        init_amp_coef,
        caller.to_account_id(),
        trade_fee,
        protocol_fee,
        Some(fee_receiver()),
    );

    let rated_swap = session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into();

    for token in [sazero, wazero] {
        psp22_utils::increase_allowance(
            session,
            token.into(),
            rated_swap,
            u128::MAX,
            caller.clone(),
        )
        .unwrap();
    }

    (rated_swap, sazero.into(), wazero.into(), rate_mock_address)
}

fn set_sazero_rate(
    session: &mut Session<MinimalRuntime>,
    mock_sazero_rate_contract: AccountId,
    rate: u128,
) {
    _ = handle_ink_error(
        session
            .execute(
                mock_sazero_rate_contract::Instance::from(mock_sazero_rate_contract).set_rate(rate),
            )
            .unwrap(),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/tests/test_rated_pool.rs#L27
#[drink::test]
fn test_01(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let initial_token_supply: u128 = 1_000_000_000;
    let (rated_swap, sazero, wazero, mock_sazero_rate) = setup_rated_swap_with_tokens(
        &mut session,
        BOB,
        initial_token_supply,
        10000,
        EXPIRE_TS,
        25,
        2000,
    );

    set_timestamp(&mut session, now * EXPIRE_TS);
    set_sazero_rate(&mut session, mock_sazero_rate, 2 * RATE_PRECISION);

    _ = stable_swap::add_liquidity(
        &mut session,
        rated_swap.into(),
        BOB,
        1,
        vec![50000 * ONE_SAZERO, 100000 * ONE_WAZERO],
        bob(),
    )
    .expect("Should successfully swap. Err: {err:?}");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, bob()),
        200000 * ONE_LPT,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) = share_price_and_total_shares(
        &mut session,
        rated_swap,
        Some(vec![2 * RATE_PRECISION, RATE_PRECISION]),
    );
    assert_eq!(
        last_total_shares,
        200000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");

    transfer_and_increase_allowance(
        &mut session,
        rated_swap,
        vec![sazero, wazero],
        CHARLIE,
        vec![100000 * ONE_SAZERO, 100000 * ONE_WAZERO],
        BOB,
    );
    _ = stable_swap::add_liquidity(
        &mut session,
        rated_swap.into(),
        CHARLIE,
        1,
        vec![50000 * ONE_SAZERO, 100000 * ONE_WAZERO],
        charlie(),
    )
    .expect("Should successfully swap");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, charlie()),
        200000 * ONE_LPT,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) = share_price_and_total_shares(
        &mut session,
        rated_swap,
        Some(vec![2 * RATE_PRECISION, RATE_PRECISION]),
    );
    assert_eq!(
        last_total_shares,
        400000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");

    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        rated_swap.into(),
        CHARLIE,
        200000 * ONE_LPT,
        vec![1 * ONE_SAZERO, 1 * ONE_WAZERO],
        charlie(),
    )
    .expect("Should successfully swap");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, charlie()),
        0,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) = share_price_and_total_shares(
        &mut session,
        rated_swap,
        Some(vec![2 * RATE_PRECISION, RATE_PRECISION]),
    );
    assert_eq!(
        last_total_shares,
        200000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");

    // let err = stable_swap::remove_liquidity_by_shares(
    //     &mut session,
    //     rated_swap.into(),
    //     BOB,
    //     200000 * ONE_LPT,
    //     vec![1 * ONE_SAZERO, 1 * ONE_WAZERO],
    //     bob(),
    // )
    // .expect_err("Should return an error");

    // assert_eq!(
    //     err,
    //     StablePoolError::MinReserve(),
    //     "Should return correct error"
    // )
}
