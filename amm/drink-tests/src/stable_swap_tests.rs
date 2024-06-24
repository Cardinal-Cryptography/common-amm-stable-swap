use crate::factory_contract;
use crate::pair_contract;
use crate::pair_contract::Pair;
use crate::router_contract;
use crate::stable_pool_contract;
use crate::utils::*;

use router_contract::Router as _;

use drink::frame_support::sp_runtime::traits::IntegerSquareRoot;
use drink::frame_support::sp_runtime::traits::Scale;
use drink::{self, runtime::MinimalRuntime, session::Session};
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ToAccountId};

const LPT_DEC: u8 = 18;
const STABLE_6: u8 = 6;
const STABLE_15: u8 = 15;

const ONE_LPT: u128 = 10u128.pow(LPT_DEC as u32);
const ONE_STABLE_6: u128 = 10u128.pow(STABLE_6 as u32);
const ONE_STABLE_15: u128 = 10u128.pow(STABLE_15 as u32);

const INIT_SUPPLY: u128 = 1_002_137; // 1M

// Fees in BPS
// const TRADE_FEE: u128 = 6;
// const ADMIN_FEE: u128 = 2000;
// const BPS_DENOM: u128 = 10000;

fn setup_stable_swap(
    session: &mut Session<MinimalRuntime>,
    stable_6: AccountId,
    stable_15: AccountId,
    init_amp_coef: u128,
    caller: drink::AccountId32,
    fee_receiver: Option<AccountId>,
) -> stable_pool_contract::Instance {
    session
        .upload_code(stable_pool_contract::upload())
        .expect("Upload stable_stable_pair_contract code");
    let _ = session.set_actor(caller.clone());

    let instance = stable_pool_contract::Instance::new_stable(
        vec![stable_6, stable_15],
        vec![STABLE_6, STABLE_15],
        init_amp_coef,
        caller.to_account_id(),
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
    amp_coef: u128,
) -> (AccountId, AccountId, AccountId) {
    upload_all(session);

    let stable_6 = psp22_utils::setup_with_amounts(
        session,
        "stable6".to_string(),
        STABLE_6,
        INIT_SUPPLY,
        BOB,
    );
    let stable_15 = psp22_utils::setup_with_amounts(
        session,
        "stable18".to_string(),
        STABLE_15,
        INIT_SUPPLY,
        BOB,
    );
    let stable_pool_contract = setup_stable_swap(
        session,
        stable_6.into(),
        stable_15.into(),
        amp_coef,
        BOB,
        Some(bob()),
    );

    for token in [stable_6, stable_15] {
        psp22_utils::increase_allowance(
            session,
            token.into(),
            stable_pool_contract.into(),
            u128::MAX,
            BOB,
        )
        .unwrap();
    }

    (stable_pool_contract.into(), stable_6.into(), stable_15.into())
}

#[drink::test]
fn stable_test_balances_after_swap_exact_in_01(mut session: Session) {
    upload_all(&mut session);
    let (stable_swap, stable_6, stable_15) = setup_all(&mut session, 1000);
    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap.into(),
        BOB,
        1,
        vec![100_000 * ONE_STABLE_6, 100_000 * ONE_STABLE_15],
        bob(),
    );;

    let (amount_out, fee) = stable_swap::swap_exact_in(
        &mut session,
        stable_swap.into(),
        BOB,
        stable_15.into(),       // in
        stable_6.into(),        // out
        10_000 * ONE_STABLE_15,  // amount_in
        1,                      // min_token_out
        bob(),
    ).result.unwrap().unwrap();

    //check swap result, total 9999495232 including fee 0.06%
    assert_eq!(amount_out, 9993495535, "Amount out mismatch");
    assert_eq!(fee, 5999697, "Fee mismatch");

    // check if reserves are ok
    let reserves = stable_swap::reserves(
        &mut session,
        stable_swap.into(),
    );
    let balance_0 = psp22_utils::balance_of(
        &mut session,
        stable_6.into(),
        stable_swap.into()
    );
    let balance_1 = psp22_utils::balance_of(
        &mut session,
        stable_15.into(),
        stable_swap.into()
    );
    assert_eq!(reserves, vec![balance_0, balance_1], "Balances - reserves mismatch");
}