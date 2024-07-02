use crate::*;

use anyhow::Result;
use drink::{runtime::MinimalRuntime, session::Session, AccountId32};
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ContractResult, InkLangError, ToAccountId};

pub const ICE: &str = "ICE";
pub const WOOD: &str = "WOOD";
pub const FIRE: &str = "FIRE";

pub const BOB: drink::AccountId32 = AccountId32::new([1u8; 32]);
pub const CHARLIE: drink::AccountId32 = AccountId32::new([3u8; 32]);

pub const TOKEN: u128 = 10u128.pow(18);

pub fn bob() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&BOB).clone().into()
}

pub fn charlie() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&CHARLIE).clone().into()
}

pub fn upload_all(session: &mut Session<MinimalRuntime>) {
    session
        .upload_code(stable_pool_contract::upload())
        .expect("Upload stable_pool_contract code");
}

pub mod stable_swap {
    use super::*;
    use stable_pool_contract::{StablePool as _, StablePoolError, StablePoolView as _};

    pub fn setup(
        session: &mut Session<MinimalRuntime>,
        tokens: Vec<AccountId>,
        tokens_decimals: Vec<u8>,
        init_amp_coef: u128,
        caller: drink::AccountId32,
        trade_fee_bps: u16,
        protocol_fee_bps: u16,
        fee_receiver: Option<AccountId>,
    ) -> stable_pool_contract::Instance {
        let _ = session.set_actor(caller.clone());
        let instance = stable_pool_contract::Instance::new_stable(
            tokens,
            tokens_decimals,
            init_amp_coef,
            caller.to_account_id(),
            trade_fee_bps,
            protocol_fee_bps,
            fee_receiver,
        );

        session
            .instantiate(instance)
            .unwrap()
            .result
            .to_account_id()
            .into()
    }

    pub fn add_liquidity(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        caller: drink::AccountId32,
        min_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
    ) -> ContractResult<Result<Result<(u128, u128), StablePoolError>, InkLangError>> {
        let _ = session.set_actor(caller);
        session
            .execute(
                stable_pool_contract::Instance::from(stable_pool).add_liquidity(
                    min_share_amount,
                    amounts,
                    to,
                ),
            )
            .unwrap()
    }

    pub fn remove_liquidity_by_amounts(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        caller: drink::AccountId32,
        max_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
    ) -> ContractResult<Result<Result<(u128, u128), StablePoolError>, InkLangError>> {
        let _ = session.set_actor(caller);
        session
            .execute(
                stable_pool_contract::Instance::from(stable_pool).remove_liquidity_by_amounts(
                    max_share_amount,
                    amounts,
                    to,
                ),
            )
            .unwrap()
    }

    pub fn remove_liquidity_by_shares(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        caller: drink::AccountId32,
        shares_amount: u128,
        min_amounts: Vec<u128>,
        to: AccountId,
    ) -> ContractResult<Result<Result<Vec<u128>, StablePoolError>, InkLangError>> {
        let _ = session.set_actor(caller);
        session
            .execute(
                stable_pool_contract::Instance::from(stable_pool).remove_liquidity_by_shares(
                    shares_amount,
                    min_amounts,
                    to,
                ),
            )
            .unwrap()
    }

    pub fn swap_exact_in(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        caller: drink::AccountId32,
        token_in: AccountId,
        token_out: AccountId,
        token_in_amount: u128,
        min_token_out_amount: u128,
        to: AccountId,
    ) -> ContractResult<
        Result<Result<(u128, u128), StablePoolError>, ink_wrapper_types::InkLangError>,
    > {
        let _ = session.set_actor(caller);
        session
            .execute(
                stable_pool_contract::Instance::from(stable_pool).swap_exact_in(
                    token_in,
                    token_out,
                    token_in_amount,
                    min_token_out_amount,
                    to,
                ),
            )
            .unwrap()
    }

    pub fn swap_exact_out(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        caller: drink::AccountId32,
        token_in: AccountId,
        token_out: AccountId,
        token_out_amount: u128,
        max_token_in_amount: u128,
        to: AccountId,
    ) -> ContractResult<
        Result<Result<(u128, u128), StablePoolError>, ink_wrapper_types::InkLangError>,
    > {
        let _ = session.set_actor(caller);
        session
            .execute(
                stable_pool_contract::Instance::from(stable_pool).swap_exact_out(
                    token_in,
                    token_out,
                    token_out_amount,
                    max_token_in_amount,
                    to,
                ),
            )
            .unwrap()
    }

    pub fn reserves(session: &mut Session<MinimalRuntime>, stable_pool: AccountId) -> Vec<u128> {
        session
            .query(stable_pool_contract::Instance::from(stable_pool).reserves())
            .unwrap()
            .result
            .unwrap()
    }
}

pub mod psp22_utils {
    use super::*;
    use psp22::{Instance as PSP22, PSP22 as _};

    /// Uploads and creates a PSP22 instance with 1B*10^18 issuance and given names.
    /// Returns its AccountId casted to PSP22 interface.
    pub fn setup(
        session: &mut Session<MinimalRuntime>,
        name: String,
        caller: drink::AccountId32,
    ) -> psp22::Instance {
        let _code_hash = session.upload_code(psp22::upload()).unwrap();

        let _ = session.set_actor(caller);

        let instance = PSP22::new(
            1_000_000_000u128 * TOKEN,
            Some(name.clone()),
            Some(name),
            18,
        );

        session
            .instantiate(instance)
            .unwrap()
            .result
            .to_account_id()
            .into()
    }

    pub fn setup_with_amounts(
        session: &mut Session<MinimalRuntime>,
        name: String,
        decimals: u8,
        init_supply: u128,
        caller: drink::AccountId32,
    ) -> psp22::Instance {
        let _code_hash = session.upload_code(psp22::upload()).unwrap();

        let _ = session.set_actor(caller);

        let instance = PSP22::new(init_supply, Some(name.clone()), Some(name), decimals);

        session
            .instantiate(instance)
            .unwrap()
            .result
            .to_account_id()
            .into()
    }

    /// Increases allowance of given token to given spender by given amount.
    pub fn increase_allowance(
        session: &mut Session<MinimalRuntime>,
        token: AccountId,
        spender: AccountId,
        amount: u128,
        caller: drink::AccountId32,
    ) -> Result<(), psp22::PSP22Error> {
        let _ = session.set_actor(caller);

        handle_ink_error(
            session
                .execute(PSP22::increase_allowance(&token.into(), spender, amount))
                .unwrap(),
        )
    }

    /// Increases allowance of given token to given spender by given amount.
    pub fn transfer(
        session: &mut Session<MinimalRuntime>,
        token: AccountId,
        to: AccountId,
        amount: u128,
        caller: drink::AccountId32,
    ) -> Result<(), psp22::PSP22Error> {
        let _ = session.set_actor(caller);

        handle_ink_error(
            session
                .execute(PSP22::transfer(&token.into(), to, amount, [].to_vec()))
                .unwrap(),
        )
    }

    /// Returns balance of given token for given account.
    /// Fails if anything other than success.
    pub fn balance_of(
        session: &mut Session<MinimalRuntime>,
        token: AccountId,
        account: AccountId,
    ) -> u128 {
        handle_ink_error(
            session
                .query(PSP22::balance_of(&token.into(), account))
                .unwrap(),
        )
    }

    pub fn total_supply(session: &mut Session<MinimalRuntime>, token: AccountId) -> u128 {
        handle_ink_error(session.query(PSP22::total_supply(&token.into())).unwrap())
    }
}

pub fn get_timestamp(session: &mut Session<MinimalRuntime>) -> u64 {
    session.sandbox().get_timestamp()
}

pub fn set_timestamp(session: &mut Session<MinimalRuntime>, timestamp: u64) {
    session.sandbox().set_timestamp(timestamp);
}

pub fn handle_ink_error<R>(res: ContractResult<Result<R, InkLangError>>) -> R {
    match res.result {
        Err(ink_lang_err) => panic!("InkLangError: {:?}", ink_lang_err),
        Ok(r) => r,
    }
}
