use crate::factory_contract;
use crate::pair_contract::Pair;
use crate::router_contract;
use crate::stable_pool_contract;
use crate::utils::*;

use factory_contract::Factory as _;
use router_contract::Router as _;
use stable_pool_contract::StablePool as _;

use drink::frame_support::sp_runtime::traits::IntegerSquareRoot;
use drink::frame_support::sp_runtime::traits::Scale;
use drink::{self, runtime::MinimalRuntime, session::Session};
use ink_primitives::AccountId;
use ink_wrapper_types::Connection;

const ICE_DEC: u8 = 6;
const WOOD_DEC: u8 = 12;
const FIRE_DEC: u8 = 18;

const ONE_ICE: u128 = 10u128.pow(ICE_DEC as u32);
const ONE_WOOD: u128 = 10u128.pow(WOOD_DEC as u32);
const ONE_FIRE: u128 = 10u128.pow(FIRE_DEC as u32);

const INIT_SUPPLY: u128 = 1_000_000; // 1M

const RUNS: u64 = 20;

fn setup_test_contracts_2pool(
    session: &mut Session<MinimalRuntime>,
    enable_admin_fee: bool,
) -> (AccountId, AccountId, AccountId, AccountId, AccountId) {
    upload_all(session);

    let fee_to_setter = bob();
    let factory = factory::setup(session, fee_to_setter);
    if enable_admin_fee {
        let _ = session.set_actor(BOB);
        _ = session.execute(
            factory_contract::Instance::from(factory).set_fee_to(AccountId::from([42u8; 32])),
        );
    }

    let fee_receiver = if enable_admin_fee { Some(bob()) } else { None };

    let wazero = wazero::setup(session);
    let ice = psp22_utils::setup_with_amounts(
        session,
        ICE.to_string(),
        ICE_DEC,
        INIT_SUPPLY * ONE_ICE,
        BOB,
    );
    let wood = psp22_utils::setup_with_amounts(
        session,
        WOOD.to_string(),
        WOOD_DEC,
        INIT_SUPPLY * ONE_WOOD,
        BOB,
    );
    let stable_swap = stable_swap::setup(
        session,
        vec![ice.into(), wood.into()],
        vec![ICE_DEC, WOOD_DEC],
        100, // A = 100
        BOB,
        fee_receiver,
    );

    let router = router::setup(session, factory.into(), wazero.into());

    for token in [ice, wood] {
        psp22_utils::increase_allowance(session, token.into(), stable_swap.into(), u128::MAX, BOB)
            .unwrap();
        psp22_utils::increase_allowance(session, token.into(), router.into(), u128::MAX, BOB)
            .unwrap();
    }
    (
        factory.into(),
        router.into(),
        stable_swap.into(),
        ice.into(),
        wood.into(),
    )
}

#[drink::test]
#[cfg_attr(not(feature = "benchmark"), ignore)]
fn benchmark_2pool_mint_liquidity_imbalanced(&mut session: Session) {
    let (_factory, _router, stable_swap, ice, wood) =
        setup_test_contracts_2pool(&mut session, false);
    let (mut total_rt, mut total_ps) = (0u64, 0u64);
    let imbalance: u128 = 1_000; // 1k
    for i in 0..RUNS {
        let res = stable_swap::add_liquidity(
            &mut session,
            stable_swap.into(),
            BOB,
            1,
            vec![
                INIT_SUPPLY * ONE_ICE / (RUNS as u128),
                (INIT_SUPPLY - ((i as u128) * imbalance)) * ONE_WOOD / (RUNS as u128),
            ],
            bob(),
        );
        total_rt += res.gas_required.ref_time();
        total_ps += res.gas_required.proof_size();
        let handled_res = handle_ink_error(res);
        assert!(handled_res.is_ok(), "Error: {handled_res:?}");
    }
    let av_rt = total_rt / RUNS;
    let av_ps = total_ps / RUNS;
    println!("\x1b[0;36m2POOL: \x1b[1;36mMint Liquidity Imbalanced");
    println!("\x1b[0;34mAverages over {RUNS:?} runs:");
    println!("\x1b[0;33mRefTime     : \x1b[0;33m{av_rt:?}");
    println!("\x1b[0;33mProofSize   : \x1b[0;33m{av_ps:?}");
    println!("\x1b[0;0m");
}

#[drink::test]
#[cfg_attr(not(feature = "benchmark"), ignore)]
fn benchmark_2pool_burn_liquidity_imbalanced(&mut session: Session) {
    let (_factory, _router, stable_swap, ice, wood) =
        setup_test_contracts_2pool(&mut session, false);
    let (mut total_rt, mut total_ps) = (0u64, 0u64);
    let imbalance: u128 = 1_000; // 1k
    stable_swap::add_liquidity(
        &mut session,
        stable_swap.into(),
        BOB,
        1,
        vec![INIT_SUPPLY * ONE_ICE, INIT_SUPPLY * ONE_WOOD],
        bob(),
    );
    for i in 0..RUNS {
        let res = stable_swap::remove_liquidity_by_amounts(
            &mut session,
            stable_swap.into(),
            BOB,
            u128::MAX,
            vec![
                ((INIT_SUPPLY - 1) * ONE_ICE) / (RUNS as u128),
                (INIT_SUPPLY - ((i as u128) * imbalance)) * ONE_WOOD / (RUNS as u128),
            ],
            bob(),
        );
        total_rt += res.gas_required.ref_time();
        total_ps += res.gas_required.proof_size();
        let handled_res = handle_ink_error(res);
        assert!(handled_res.is_ok(), "Error: {handled_res:?}");
    }
    let av_rt = total_rt / RUNS;
    let av_ps = total_ps / RUNS;
    println!("\x1b[0;36m2POOL: \x1b[1;36mBurn Liquidity Imbalanced");
    println!("\x1b[0;34mAverages over {RUNS:?} runs:");
    println!("\x1b[0;33mRefTime     : \x1b[0;33m{av_rt:?}");
    println!("\x1b[0;33mProofSize   : \x1b[0;33m{av_ps:?}");
    println!("\x1b[0;0m");
}

#[drink::test]
#[cfg_attr(not(feature = "benchmark"), ignore)]
fn benchmark_2pool_swap(&mut session: Session) {
    let (_factory, _router, stable_swap, ice, wood) =
        setup_test_contracts_2pool(&mut session, false);
    let (mut total_rt, mut total_ps) = (0u64, 0u64);
    stable_swap::add_liquidity(
        &mut session,
        stable_swap.into(),
        BOB,
        1,
        vec![INIT_SUPPLY * ONE_ICE / 10, INIT_SUPPLY * ONE_WOOD / 10],
        bob(),
    );
    let swap_amount = 1_000u128;
    for i in 0..RUNS {
        let res = stable_swap::swap_exact_in(
            &mut session,
            stable_swap.into(),
            BOB,
            ice,
            wood,
            swap_amount * ONE_ICE, // token in
            1,                     // min_token_out
            bob(),
        );
        total_rt += res.gas_required.ref_time();
        total_ps += res.gas_required.proof_size();
        let handled_res = handle_ink_error(res);
        assert!(handled_res.is_ok(), "Error: {handled_res:?}");
    }
    let av_rt = total_rt / RUNS;
    let av_ps = total_ps / RUNS;
    println!("\x1b[0;36m2POOL: \x1b[1;36mSwap");
    println!("\x1b[0;34mAverages over {RUNS:?} runs:");
    println!("\x1b[0;33mRefTime     : \x1b[0;33m{av_rt:?}");
    println!("\x1b[0;33mProofSize   : \x1b[0;33m{av_ps:?}");
    println!("\x1b[0;0m");
}

#[drink::test]
#[cfg_attr(not(feature = "benchmark"), ignore)]
fn benchmark_router_add_liqudity(&mut session: Session) {
    let (_factory, router, _stable_swap, ice, wood) =
        setup_test_contracts_2pool(&mut session, false);
    let (mut total_rt, mut total_ps) = (0u64, 0u64);

    let token_amount = 1_000_000u128;
    for i in 0..RUNS {
        let now = get_timestamp(&mut session);
        set_timestamp(&mut session, now);
        let deadline = now + 10;
        let res = session
            .execute(router_contract::Instance::from(router).add_liquidity(
                ice,
                wood,
                token_amount,
                token_amount,
                token_amount,
                token_amount,
                bob(),
                deadline,
            ))
            .unwrap();
        total_rt += res.gas_required.ref_time();
        total_ps += res.gas_required.proof_size();
        let handled_res = handle_ink_error(res);
        assert!(handled_res.is_ok(), "Error: {handled_res:?}");
    }
    let av_rt = total_rt / RUNS;
    let av_ps = total_ps / RUNS;
    println!("\x1b[0;36mROUTER: \x1b[1;36mAdd Liquidity");
    println!("\x1b[0;34mAverages over {RUNS:?} runs:");
    println!("\x1b[0;33mRefTime     : \x1b[0;33m{av_rt:?}");
    println!("\x1b[0;33mProofSize   : \x1b[0;33m{av_ps:?}");
    println!("\x1b[0;0m");
}

#[drink::test]
#[cfg_attr(not(feature = "benchmark"), ignore)]
fn benchmark_router_remove_liqudity(&mut session: Session) {
    let (factory, router, _stable_swap, ice, wood) =
        setup_test_contracts_2pool(&mut session, false);
    let (mut total_rt, mut total_ps) = (0u64, 0u64);

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let deadline = now + 10;
    let res = session
        .execute(router_contract::Instance::from(router).add_liquidity(
            ice,
            wood,
            INIT_SUPPLY * ONE_ICE,
            INIT_SUPPLY * ONE_WOOD,
            INIT_SUPPLY * ONE_ICE,
            INIT_SUPPLY * ONE_WOOD,
            bob(),
            deadline,
        ))
        .unwrap();

    let tot_sup_lp = handle_ink_error(res).unwrap().2;
    let token_amount = tot_sup_lp / (RUNS as u128);

    let ice_wood_pair = factory::get_pair(&mut session, factory, ice, wood);

    _ = psp22_utils::increase_allowance(
        &mut session,
        ice_wood_pair.into(),
        router.into(),
        u128::MAX,
        BOB,
    );

    for i in 0..RUNS {
        let now = get_timestamp(&mut session);
        set_timestamp(&mut session, now);
        let deadline = now + 10;
        let res = session
            .execute(router_contract::Instance::from(router).remove_liquidity(
                ice,
                wood,
                token_amount,
                ONE_ICE,
                ONE_WOOD,
                bob(),
                deadline,
            ))
            .unwrap();
        total_rt += res.gas_required.ref_time();
        total_ps += res.gas_required.proof_size();
        let handled_res = handle_ink_error(res);
        assert!(handled_res.is_ok(), "Error: {handled_res:?}");
    }
    let av_rt = total_rt / RUNS;
    let av_ps = total_ps / RUNS;
    println!("\x1b[0;36mROUTER: \x1b[1;36mRemove Liquidity");
    println!("\x1b[0;34mAverages over {RUNS:?} runs:");
    println!("\x1b[0;33mRefTime     : \x1b[0;33m{av_rt:?}");
    println!("\x1b[0;33mProofSize   : \x1b[0;33m{av_ps:?}");
    println!("\x1b[0;0m");
}

#[drink::test]
#[cfg_attr(not(feature = "benchmark"), ignore)]
fn benchmark_router_swap(&mut session: Session) {
    let (factory, router, _stable_swap, ice, wood) =
        setup_test_contracts_2pool(&mut session, false);
    let (mut total_rt, mut total_ps) = (0u64, 0u64);

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let deadline = now + 10;
    let res = session
        .execute(router_contract::Instance::from(router).add_liquidity(
            ice,
            wood,
            INIT_SUPPLY * ONE_ICE / 2,
            INIT_SUPPLY * ONE_WOOD / 2,
            INIT_SUPPLY * ONE_ICE / 2,
            INIT_SUPPLY * ONE_WOOD / 2,
            bob(),
            deadline,
        ))
        .unwrap();
    let token_amount = INIT_SUPPLY * ONE_ICE / 2 / (RUNS as u128);
    for i in 0..RUNS {
        let now = get_timestamp(&mut session);
        set_timestamp(&mut session, now);
        let deadline = now + 10;
        let res = session
            .execute(
                router_contract::Instance::from(router).swap_exact_tokens_for_tokens(
                    token_amount,
                    1,
                    vec![ice, wood],
                    bob(),
                    deadline,
                ),
            )
            .unwrap();
        total_rt += res.gas_required.ref_time();
        total_ps += res.gas_required.proof_size();
        let handled_res = handle_ink_error(res);
        assert!(handled_res.is_ok(), "Error: {handled_res:?}");
    }
    let av_rt = total_rt / RUNS;
    let av_ps = total_ps / RUNS;
    println!("\x1b[0;36mROUTER: \x1b[1;36mSwap(2-token-path)");
    println!("\x1b[0;34mAverages over {RUNS:?} runs:");
    println!("\x1b[0;33mRefTime     : \x1b[0;33m{av_rt:?}");
    println!("\x1b[0;33mProofSize   : \x1b[0;33m{av_ps:?}");
    println!("\x1b[0;0m");
}
