use crate::factory_contract;
use crate::stable_swap_contract;
use crate::utils::*;

use factory_contract::Factory as _;
use stable_swap_contract::StablePool as _;

use drink::frame_support::sp_runtime::traits::IntegerSquareRoot;
use drink::frame_support::sp_runtime::traits::Scale;
use drink::{self, session::Session};
use ink_wrapper_types::Connection;

#[drink::test]
#[cfg_attr(not(feature = "benchmark"), ignore)]
fn benchmark_mint_burn_liquidity_2_pool(&mut session: Session) {
    upload_all(&mut session);

    let fee_to_setter = bob();

    let factory = factory::setup(&mut session, fee_to_setter);
    let ice = psp22_utils::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);
    let stable_swap = stable_swap::setup(
        &mut session,
        vec![ice.into(), wood.into()],
        vec![12, 12],
        100,
        factory.into(),
        BOB,
    );

    let token_amount = 10_000;
    psp22_utils::increase_allowance(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::increase_allowance(
        &mut session,
        wood.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    let (mut lp_tokens, _) = handle_benchmark(
        stable_swap::add_liquidity(
            &mut session,
            stable_swap.into(),
            BOB,
            1,
            vec![token_amount, token_amount],
            bob(),
        ),
        "2POOL: Mint Liquidity 1",
    )
    .unwrap();
    let token_amount = 10_000;
    psp22_utils::increase_allowance(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::increase_allowance(
        &mut session,
        wood.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    let tmp = handle_benchmark(
        stable_swap::add_liquidity(
            &mut session,
            stable_swap.into(),
            BOB,
            1,
            vec![token_amount, token_amount],
            bob(),
        ),
        "2POOL: Mint Liquidity 2",
    )
    .unwrap();
    lp_tokens += tmp.0;
    psp22_utils::increase_allowance(
        &mut session,
        stable_swap.into(),
        stable_swap.into(),
        lp_tokens,
        BOB,
    )
    .unwrap();
    assert!(handle_benchmark(
        stable_swap::remove_liquidity(
            &mut session,
            stable_swap.into(),
            BOB,
            1_000_000_000_000,
            vec![token_amount, token_amount],
            bob()
        ),
        "2POOL: Burn Liquidity 1",
    )
    .is_ok());
    assert!(handle_benchmark(
        stable_swap::remove_liquidity(
            &mut session,
            stable_swap.into(),
            BOB,
            1_000_000_000_000,
            vec![token_amount, token_amount],
            bob()
        ),
        "2POOL: Burn Liquidity 2",
    )
    .is_ok());
}

#[drink::test]
#[cfg_attr(not(feature = "benchmark"), ignore)]
fn benchmark_mint_burn_liquidity_3_pool(&mut session: Session) {
    upload_all(&mut session);

    let fee_to_setter = bob();

    let factory = factory::setup(&mut session, fee_to_setter);
    let ice = psp22_utils::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);
    let fire = psp22_utils::setup(&mut session, FIRE.to_string(), BOB);
    let stable_swap = stable_swap::setup(
        &mut session,
        vec![ice.into(), wood.into(), fire.into()],
        vec![12, 12, 12],
        100,
        factory.into(),
        BOB,
    );

    let token_amount = 10_000;
    psp22_utils::increase_allowance(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::increase_allowance(
        &mut session,
        wood.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::increase_allowance(
        &mut session,
        fire.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    let (mut lp_tokens, _) = handle_benchmark(
        stable_swap::add_liquidity(
            &mut session,
            stable_swap.into(),
            BOB,
            1,
            vec![token_amount, token_amount, token_amount],
            bob(),
        ),
        "3POOL: Mint Liquidity 1",
    )
    .unwrap();
    psp22_utils::increase_allowance(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::increase_allowance(
        &mut session,
        wood.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::increase_allowance(
        &mut session,
        fire.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();

    let tmp = handle_benchmark(
        stable_swap::add_liquidity(
            &mut session,
            stable_swap.into(),
            BOB,
            1,
            vec![token_amount, token_amount, token_amount],
            bob(),
        ),
        "3POOL: Mint Liquidity 2",
    )
    .unwrap();
    lp_tokens += tmp.0;
    psp22_utils::increase_allowance(
        &mut session,
        stable_swap.into(),
        stable_swap.into(),
        lp_tokens,
        BOB,
    )
    .unwrap();
    assert!(handle_benchmark(
        stable_swap::remove_liquidity(
            &mut session,
            stable_swap.into(),
            BOB,
            1_000_000_000_000,
            vec![token_amount, token_amount, token_amount],
            bob()
        ),
        "3POOL: Burn Liquidity 1",
    )
    .is_ok());
    assert!(handle_benchmark(
        stable_swap::remove_liquidity(
            &mut session,
            stable_swap.into(),
            BOB,
            1_000_000_000_000,
            vec![token_amount, token_amount, token_amount],
            bob()
        ),
        "3POOL: Burn Liquidity 2",
    )
    .is_ok());
}

#[drink::test]
#[cfg_attr(not(feature = "benchmark"), ignore)]
fn benchmark_swap_2_pool(&mut session: Session) {
    upload_all(&mut session);

    let fee_to_setter = bob();

    let factory = factory::setup(&mut session, fee_to_setter);
    let ice = psp22_utils::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);
    let stable_swap = stable_swap::setup(
        &mut session,
        vec![ice.into(), wood.into()],
        vec![12, 12, 12],
        100,
        factory.into(),
        BOB,
    );

    let token_amount = 10_000;
    psp22_utils::increase_allowance(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::increase_allowance(
        &mut session,
        wood.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    stable_swap::add_liquidity(
        &mut session,
        stable_swap.into(),
        BOB,
        1,
        vec![token_amount, token_amount],
        bob(),
    );
    psp22_utils::increase_allowance(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount / 2,
        BOB,
    )
    .unwrap();
    assert!(handle_benchmark(
        stable_swap::swap(&mut session, stable_swap.into(), BOB, 0, 1, token_amount / 2, 1, bob()),
        "2POOL: Swap 1",
    )
    .is_ok());
    psp22_utils::increase_allowance(
        &mut session,
        wood.into(),
        stable_swap.into(),
        token_amount / 2,
        BOB,
    )
    .unwrap();
    assert!(handle_benchmark(
        stable_swap::swap(&mut session, stable_swap.into(), BOB, 1, 0, token_amount / 2, 1, bob()),
        "2POOL: Swap 2",
    )
    .is_ok());
}

#[drink::test]
#[cfg_attr(not(feature = "benchmark"), ignore)]
fn benchmark_swap_3_pool(&mut session: Session) {
    upload_all(&mut session);

    let fee_to_setter = bob();

    let factory = factory::setup(&mut session, fee_to_setter);
    let ice = psp22_utils::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);
    let fire = psp22_utils::setup(&mut session, FIRE.to_string(), BOB);
    let stable_swap = stable_swap::setup(
        &mut session,
        vec![ice.into(), wood.into(), fire.into()],
        vec![12, 12, 12],
        100,
        factory.into(),
        BOB,
    );

    let token_amount = 10_000;
    psp22_utils::increase_allowance(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::increase_allowance(
        &mut session,
        wood.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::increase_allowance(
        &mut session,
        fire.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    stable_swap::add_liquidity(
        &mut session,
        stable_swap.into(),
        BOB,
        1,
        vec![token_amount, token_amount, token_amount],
        bob(),
    );
    psp22_utils::increase_allowance(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount / 2,
        BOB,
    )
    .unwrap();
    assert!(handle_benchmark(
        stable_swap::swap(&mut session, stable_swap.into(), BOB, 0, 1, token_amount / 2, 1, bob()),
        "3POOL: Swap 1",
    )
    .is_ok());
    psp22_utils::increase_allowance(
        &mut session,
        fire.into(),
        stable_swap.into(),
        token_amount / 2,
        BOB,
    )
    .unwrap();
    assert!(handle_benchmark(
        stable_swap::swap(&mut session, stable_swap.into(), BOB, 2, 0, token_amount / 2, 1, bob()),
        "3POOL: Swap 2",
    )
    .is_ok());
}
