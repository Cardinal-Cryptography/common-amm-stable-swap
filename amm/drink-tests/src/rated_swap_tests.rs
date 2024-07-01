use crate::factory_contract;
use crate::pair_contract;
use crate::pair_contract::Pair;
use crate::router_contract;
use crate::sazero_rate_mock_contract;
use crate::stable_pool_contract;
use crate::utils::*;

use router_contract::Router as _;

use drink::frame_support::sp_runtime::traits::IntegerSquareRoot;
use drink::frame_support::sp_runtime::traits::Scale;
use drink::{self, runtime::MinimalRuntime, session::Session};
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ToAccountId};

const SAZERO_DEC: u8 = 12;
const WAZERO_DEC: u8 = 12;

const ONE_SAZERO: u128 = 10u128.pow(SAZERO_DEC as u32);
const ONE_AZERO: u128 = 10u128.pow(WAZERO_DEC as u32);

const INIT_SUPPLY: u128 = 1_000_000; // 100k

const TRADE_FEE_BPS: u16 = 6;
const PROTOCOL_FEE_BPS: u16 = 2000;

const AMP_COEF: u128 = 1000;

fn setup_rated_swap(
    session: &mut Session<MinimalRuntime>,
    sazero: AccountId,
    wazero: AccountId,
    init_amp_coef: u128,
    rate_expiration_duration_ms: u64,
    caller: drink::AccountId32,
    trade_fee: u16,
    protocol_fee: u16,
    fee_receiver: Option<AccountId>,
) -> stable_pool_contract::Instance {
    //upload and deploy rate mock
    session
        .upload_code(stable_pool_contract::upload())
        .expect("Upload stable_stable_pair_contract code");
    session
        .upload_code(sazero_rate_mock_contract::upload())
        .expect("Upload sazero_rate_mock_contract code");
    let _ = session.set_actor(caller.clone());

    let instance = sazero_rate_mock_contract::Instance::new();

    let rate_mock_address = session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into();
    let instance = stable_pool_contract::Instance::new_rated(
        vec![sazero, wazero],
        vec![SAZERO_DEC, WAZERO_DEC],
        vec![Some(rate_mock_address), None],
        rate_expiration_duration_ms,
        init_amp_coef,
        caller.to_account_id(),
        trade_fee,
        protocol_fee,
        fee_receiver,
    );

    session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into()
}

fn setup_all(
    session: &mut Session<MinimalRuntime>,
    enable_protocol_fee: bool,
) -> (AccountId, AccountId, AccountId) {
    upload_all(session);

    let wazero = psp22_utils::setup_with_amounts(
        session,
        "wAZERO".to_string(),
        WAZERO_DEC,
        INIT_SUPPLY * ONE_AZERO,
        BOB,
    );
    let sazero = psp22_utils::setup_with_amounts(
        session,
        "SAZERO".to_string(),
        SAZERO_DEC,
        INIT_SUPPLY * ONE_SAZERO,
        BOB,
    );
    let stable_pool_contract = setup_rated_swap(
        session,
        sazero.into(),
        wazero.into(),
        AMP_COEF,
        10000, //10 seconds rate cache expiration
        BOB,
        TRADE_FEE_BPS,
        PROTOCOL_FEE_BPS,
        Some(bob()),
    );

    for token in [sazero, wazero] {
        psp22_utils::increase_allowance(
            session,
            token.into(),
            stable_pool_contract.into(),
            u128::MAX,
            BOB,
        )
        .unwrap();
    }

    (stable_pool_contract.into(), sazero.into(), wazero.into())
}

#[drink::test]
fn test_rated_1(mut session: Session) {
    let one_minute: u64 = 60000;
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    upload_all(&mut session);
    let (rated_swap, sazero, wazero) = setup_all(&mut session, false);
    _ = stable_swap::add_liquidity(
        &mut session,
        rated_swap.into(),
        BOB,
        1,
        vec![INIT_SUPPLY * ONE_SAZERO / 10, INIT_SUPPLY * ONE_AZERO / 10],
        bob(),
    );
    let amount = 10_000 * ONE_SAZERO; // 10k

    psp22_utils::increase_allowance(
        &mut session,
        sazero.into(),
        rated_swap.into(),
        u128::MAX,
        BOB,
    )
    .unwrap();

    set_timestamp(&mut session, now + 10000 * one_minute);
    let (amount_out, fee) = stable_swap::swap_exact_in(
        &mut session,
        rated_swap.into(),
        BOB,
        sazero.into(),
        wazero.into(),
        amount,
        1, // min_token_out
        bob(),
    )
    .result
    .unwrap()
    .unwrap_or_else(|_| panic!("Should return valid result"));
    let reserves = stable_swap::reserves(&mut session, rated_swap.into());
    let balance_0 = psp22_utils::balance_of(&mut session, sazero.into(), rated_swap.into());
    let balance_1 = psp22_utils::balance_of(&mut session, wazero.into(), rated_swap.into());
    assert_eq!(reserves, vec![balance_0, balance_1]);
}
