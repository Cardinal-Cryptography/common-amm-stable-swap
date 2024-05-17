use crate::factory_contract;
use crate::pair_contract;
use crate::pair_contract::Pair;
use crate::router_contract;
use crate::rated_stable_pair_contract;
use crate::sazero_rate_mock_contract;
use crate::utils::*;

use factory_contract::Factory as _;
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

const INIT_SUPPLY: u128 = 1_000_000; // 1M

const AMP_COEF: u128 = 1000;

fn setup_rated_swap(
    session: &mut Session<MinimalRuntime>,
    sazero: AccountId,
    wazero: AccountId,
    init_amp_coef: u128,
    factory: AccountId,
    caller: drink::AccountId32,
) -> rated_stable_pair_contract::Instance {
    //upload and deploy rate mock
    session
        .upload_code(rated_stable_pair_contract::upload())
        .expect("Upload rated_stable_pair_contract code");
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

    let instance = rated_stable_pair_contract::Instance::new(
        sazero,
        wazero,
        SAZERO_DEC,
        WAZERO_DEC,
        rate_mock_address,
        init_amp_coef,
        factory,
        caller.to_account_id(),
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
    enable_admin_fee: bool,
) -> (AccountId, AccountId, AccountId) {
    upload_all(session);
    let fee_to_setter = bob();
    let factory = factory::setup(session, fee_to_setter);
    if enable_admin_fee {
        let _ = session.set_actor(BOB);
        _ = session.execute(
            factory_contract::Instance::from(factory).set_fee_to(AccountId::from([42u8; 32])),
        );
    }

    let wazero = psp22_utils::setup_with_amounts(
        session,
        "wAZERO".to_string(),
        WAZERO_DEC,
        INIT_SUPPLY,
        BOB,
    );
    let sazero = psp22_utils::setup_with_amounts(
        session,
        "SAZERO".to_string(),
        SAZERO_DEC,
        INIT_SUPPLY,
        BOB,
    );
    let rated_swap_contract = setup_rated_swap(
        session,
        sazero.into(),
        wazero.into(),
        AMP_COEF,
        factory.into(),
        BOB,
    );

    for token in [sazero, wazero] {
        psp22_utils::increase_allowance(
            session,
            token.into(),
            rated_swap_contract.into(),
            u128::MAX,
            BOB,
        )
        .unwrap();
    }

    (rated_swap_contract.into(), sazero.into(), wazero.into())
}

#[drink::test]
fn rated_test_1(mut session: Session) {
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
        vec![INIT_SUPPLY * ONE_SAZERO / 2, INIT_SUPPLY * ONE_AZERO / 2],
        bob(),
    );
    let amount = 10 * ONE_SAZERO;
    psp22_utils::transfer(
        &mut session,
        sazero.into(),
        rated_swap.into(),
        amount,
        BOB,
    )
    .unwrap();
    set_timestamp(&mut session, now + 10000 * one_minute);
    let res = stable_swap::swap_excess(
        &mut session,
        rated_swap.into(),
        BOB,
        sazero.into(),
        wazero.into(),
        1,                     // min_token_out
        bob(),
    ).result;
    let reserves = stable_swap::reserves(
        &mut session,
        rated_swap.into(),
    );
    let balance_0 = psp22_utils::balance_of(
        &mut session,
        sazero.into(),
        rated_swap.into()
    );
    let balance_1 = psp22_utils::balance_of(
        &mut session,
        wazero.into(),
        rated_swap.into()
    );
    assert_eq!(reserves, vec![balance_0, balance_1]);
}