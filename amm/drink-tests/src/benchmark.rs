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
    psp22_utils::transfer(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::transfer(
        &mut session,
        wood.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    let mut lp_tokens = handle_benchmark(
        stable_swap::mint_liquidity(&mut session, stable_swap.into(), bob(), BOB),
        "2POOL: Mint Liquidity 1",
    )
    .unwrap();
    let token_amount = 10_000;
    psp22_utils::transfer(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::transfer(
        &mut session,
        wood.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    lp_tokens += handle_benchmark(
        stable_swap::mint_liquidity(&mut session, stable_swap.into(), bob(), BOB),
        "2POOL: Mint Liquidity 2",
    )
    .unwrap();
    psp22_utils::transfer(
        &mut session,
        stable_swap.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    assert!(handle_benchmark(
        stable_swap::burn_liquidity(&mut session, stable_swap.into(), bob(), None, BOB),
        "2POOL: Burn Liquidity Equal",
    )
    .is_ok());
    psp22_utils::transfer(
        &mut session,
        stable_swap.into(),
        stable_swap.into(),
        lp_tokens - token_amount,
        BOB,
    )
    .unwrap();
    assert!(handle_benchmark(
        stable_swap::burn_liquidity(
            &mut session,
            stable_swap.into(),
            bob(),
            Some(vec![20, 30]),
            BOB,
        ),
        "2POOL: Burn Liquidity Not Equal",
    )
    .is_ok());
}

#[drink::test]
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
    psp22_utils::transfer(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::transfer(
        &mut session,
        wood.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::transfer(
        &mut session,
        fire.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    let mut lp_tokens = handle_benchmark(
        stable_swap::mint_liquidity(&mut session, stable_swap.into(), bob(), BOB),
        "3POOL: Mint Liquidity 1",
    )
    .unwrap();
    psp22_utils::transfer(
        &mut session,
        ice.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::transfer(
        &mut session,
        wood.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    psp22_utils::transfer(
        &mut session,
        fire.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    lp_tokens += handle_benchmark(
        stable_swap::mint_liquidity(&mut session, stable_swap.into(), bob(), BOB),
        "3POOL: Mint Liquidity 2",
    )
    .unwrap();
    psp22_utils::transfer(
        &mut session,
        stable_swap.into(),
        stable_swap.into(),
        token_amount,
        BOB,
    )
    .unwrap();
    assert!(handle_benchmark(
        stable_swap::burn_liquidity(&mut session, stable_swap.into(), bob(), None, BOB),
        "3POOL: Burn Liquidity Equal",
    )
    .is_ok());
    psp22_utils::transfer(
        &mut session,
        stable_swap.into(),
        stable_swap.into(),
        lp_tokens - token_amount,
        BOB,
    )
    .unwrap();
    assert!(handle_benchmark(
        stable_swap::burn_liquidity(
            &mut session,
            stable_swap.into(),
            bob(),
            Some(vec![20, 30, 50]),
            BOB,
        ),
        "3POOL: Burn Liquidity Not Equal",
    )
    .is_ok());
}
