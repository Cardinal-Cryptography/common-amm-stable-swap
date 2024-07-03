mod tests_add_remove_lp;
mod tests_rated;
mod tests_swap_exact_in;
mod tests_swap_received;

use crate::stable_pool_contract;
pub use crate::utils::*;
use primitive_types::U256;

pub use stable_pool_contract::StablePool as _;
pub use stable_pool_contract::StablePoolError;
pub use stable_pool_contract::StablePoolView as _;

use drink::{self, runtime::MinimalRuntime, session::Session, AccountId32};

use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ToAccountId};

pub const FEE_BPS_DENOM: u128 = 10_000;

pub const FEE_RECEIVER: AccountId32 = AccountId32::new([42u8; 32]);

const RATE_PRECISION: u128 = 10u128.pow(12);

pub fn fee_receiver() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&FEE_RECEIVER).clone().into()
}

pub fn setup_stable_swap_with_tokens(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    token_supply: Vec<u128>,
    amp_coef: u128,
    fee_bps: u16,
    protocol_fee_bps: u16,
    caller: AccountId32,
) -> (AccountId, Vec<AccountId>) {
    let _ = session.set_actor(caller);

    if token_decimals.len() != token_supply.len() {
        panic!("SETUP: Inconsistent number of tokens.")
    }

    upload_all(session);

    // instantiate tokens
    let tokens: Vec<AccountId> = token_decimals
        .iter()
        .zip(token_supply.iter())
        .enumerate()
        .map(|(id, (&decimals, &supply))| {
            psp22_utils::setup_with_amounts(
                session,
                format!("Test Token {id}").to_string(),
                decimals,
                supply,
                BOB,
            )
            .into()
        })
        .collect::<Vec<AccountId>>();

    // instantiate stable_swap
    let instance = stable_pool_contract::Instance::new_stable(
        tokens.clone(),
        token_decimals,
        amp_coef,
        bob(),
        fee_bps,
        protocol_fee_bps,
        Some(fee_receiver()),
    );

    let stable_swap: stable_pool_contract::Instance = session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into();

    // setup max allowance for stable swap contract on both tokens
    for token in tokens.clone() {
        psp22_utils::increase_allowance(session, token.into(), stable_swap.into(), u128::MAX, BOB)
            .unwrap();
    }

    (stable_swap.into(), tokens)
}

pub fn share_price_and_total_shares(
    session: &mut Session<MinimalRuntime>,
    stable_swap: AccountId,
    token_rates: Option<Vec<u128>>,
) -> (u128, u128) {
    let total_shares = psp22_utils::total_supply(session, stable_swap);
    let reserves = stable_swap::reserves(session, stable_swap);
    let token_rates: Vec<u128> = if let Some(rates) = token_rates {
        rates
    } else {
        reserves.iter().map(|_| RATE_PRECISION).collect()
    };
    let sum_token = stable_swap::tokens(session, stable_swap)
        .iter()
        .zip(reserves.iter())
        .zip(token_rates.iter())
        .fold(0, |acc, ((&token, reserve), rate)| {
            acc + reserve
                * 10u128.pow((18 - psp22_utils::token_decimals(session, token)).into())
                * rate
                / RATE_PRECISION
        });

    (
        U256::from(sum_token)
            .checked_mul(100000000.into())
            .unwrap()
            .checked_div(total_shares.into())
            .unwrap_or(100000000.into())
            .as_u128(),
        total_shares,
    )
}

pub fn transfer_and_increase_allowance(
    session: &mut Session<MinimalRuntime>,
    stable_swap: AccountId,
    tokens: Vec<AccountId>,
    receiver: AccountId32,
    amounts: Vec<u128>,
    caller: AccountId32,
) {
    for (&token, &amount) in tokens.iter().zip(amounts.iter()) {
        _ = psp22_utils::transfer(
            session,
            token,
            receiver.to_account_id(),
            amount,
            caller.clone(),
        );
        _ = psp22_utils::increase_allowance(
            session,
            token,
            stable_swap,
            u128::MAX,
            receiver.clone(),
        );
    }
}
