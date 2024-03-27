#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod amp_coef;
mod fees;
pub mod math;

#[ink::contract]
pub mod stable_pool {
    // 0.0006 * amount
    pub const TRADE_FEE_BPS: u32 = 6;
    // 0.0006 * 0.2 * amount (part of the TRADE_FEE)
    pub const ADMIN_FEE_BPS: u32 = 2_000;

    /// The target number of decimals for comparable amounts
    pub const TARGET_DECIMAL: u8 = 12;
    /// The max number of supported token decimals
    pub const MAX_DECIMAL: u8 = 18;

    pub use crate::amp_coef::*;
    pub use crate::fees::*;
    pub use crate::math;
    use amm_helpers::ensure;
    use ink::contract_ref;
    use ink::prelude::{
        string::{String, ToString},
        {vec, vec::Vec},
    };
    use psp22::{PSP22Data, PSP22Error, PSP22Event, PSP22Metadata, PSP22};
    use traits::{Factory, MathError, StablePool, StablePoolError};

    #[ink(event)]
    #[derive(Debug)]
    #[cfg_attr(feature = "std", derive(Eq, PartialEq))]
    pub struct Approval {
        /// Account providing allowance.
        #[ink(topic)]
        pub owner: AccountId,
        /// Allowance beneficiary.
        #[ink(topic)]
        pub spender: AccountId,
        /// New allowance amount.
        pub amount: u128,
    }

    /// Event emitted when transfer of tokens occurs.
    #[ink(event)]
    #[derive(Debug)]
    #[cfg_attr(feature = "std", derive(Eq, PartialEq))]
    pub struct Transfer {
        /// Transfer sender. `None` in case of minting new tokens.
        #[ink(topic)]
        pub from: Option<AccountId>,
        /// Transfer recipient. `None` in case of burning tokens.
        #[ink(topic)]
        pub to: Option<AccountId>,
        /// Amount of tokens transferred (or minted/burned).
        pub value: u128,
    }

    #[ink::storage_item]
    #[derive(Debug)]
    pub struct StablePoolData {
        /// Factory contract
        factory: contract_ref!(Factory),
        /// Tokens.
        tokens: Vec<AccountId>,
        /// Tokens.
        tokens_decimals: Vec<u8>,
        /// Reserves in comparable amounts.
        reserves: Vec<u128>,
        /// Amplification coefficient.
        amp_coef: AmplificationCoefficient,
        /// Fees
        fees: Fees,
    }

    #[ink(storage)]
    pub struct StablePoolContract {
        admin: AccountId,
        psp22: PSP22Data,
        pool: StablePoolData,
    }

    impl StablePoolContract {
        #[ink(constructor)]
        pub fn new(
            tokens: Vec<AccountId>,
            tokens_decimals: Vec<u8>,
            init_amp_coef: u128,
            factory: AccountId,
            admin: AccountId,
        ) -> Self {
            let reserves = vec![0; tokens.len()];
            Self {
                psp22: PSP22Data::default(),
                pool: StablePoolData {
                    factory: factory.into(),
                    tokens,
                    tokens_decimals,
                    reserves,
                    amp_coef: AmplificationCoefficient::new(init_amp_coef),
                    fees: Fees::new(TRADE_FEE_BPS, ADMIN_FEE_BPS),
                },
                admin,
            }
        }

        #[ink(constructor)]
        pub fn new_checked(
            tokens: Vec<AccountId>,
            tokens_decimals: Vec<u8>,
            init_amp_coef: u128,
            factory: AccountId,
            admin: AccountId,
        ) -> Result<Self, StablePoolError> {
            let mut unique_tokens = tokens.clone();
            unique_tokens.sort();
            unique_tokens.dedup();
            ensure!(
                unique_tokens.len() == tokens.len(),
                StablePoolError::IdenticalTokenId
            );
            ensure!(
                tokens.len() == tokens_decimals.len(),
                StablePoolError::IncorrectTokenCount
            );
            for &decimals in &tokens_decimals {
                ensure!(decimals <= MAX_DECIMAL, StablePoolError::TokenDecimals);
            }
            ensure!(init_amp_coef >= MIN_AMP, StablePoolError::AmpCoefToLow);
            ensure!(init_amp_coef <= MAX_AMP, StablePoolError::AmpCoefToHigh);
            Ok(Self::new(
                tokens,
                tokens_decimals,
                init_amp_coef,
                factory,
                admin,
            ))
        }

        /// A helper function emitting events contained in a vector of PSP22Events.
        fn emit_events(&self, events: Vec<PSP22Event>) {
            for event in events {
                match event {
                    PSP22Event::Transfer { from, to, value } => {
                        self.env().emit_event(Transfer { from, to, value })
                    }
                    PSP22Event::Approval {
                        owner,
                        spender,
                        amount,
                    } => self.env().emit_event(Approval {
                        owner,
                        spender,
                        amount,
                    }),
                }
            }
        }

        /// A helper funciton for getting PSP22 contract ref
        fn token_by_id(&self, id: u8) -> Result<contract_ref!(PSP22), StablePoolError> {
            Ok((*self
                .pool
                .tokens
                .get(id as usize)
                .ok_or(StablePoolError::InvalidTokenId(id))?)
            .into())
        }

        /// A helper funciton for getting PSP22 contract ref
        fn token_by_address(&self, address: AccountId) -> contract_ref!(PSP22) {
            address.into()
        }

        fn fee_to(&self) -> Option<AccountId> {
            self.pool.factory.fee_to()
        }

        /// Converts provided token `amount` to comparable amount
        fn to_comperable_amount(&self, token_id: usize, amount: u128) -> Result<u128, MathError> {
            let token_decimals = self.pool.tokens_decimals[token_id];
            if TARGET_DECIMAL == token_decimals {
                Ok(amount)
            } else if TARGET_DECIMAL > token_decimals {
                (10u128)
                    .pow(TARGET_DECIMAL.checked_sub(token_decimals).unwrap().into())
                    .checked_mul(amount)
                    .ok_or(MathError::MulOverflow(1))
            } else {
                amount
                    .checked_div(
                        (10u128).pow(token_decimals.checked_sub(TARGET_DECIMAL).unwrap().into()),
                    )
                    .ok_or(MathError::DivByZero(1))
            }
        }

        /// Converts provided comparable `amount` to token amount
        fn to_token_amount(&self, token_id: usize, amount: u128) -> Result<u128, MathError> {
            let token_decimals = self.pool.tokens_decimals[token_id];
            if TARGET_DECIMAL == token_decimals {
                Ok(amount)
            } else if TARGET_DECIMAL > token_decimals {
                amount
                    .checked_div(
                        (10u128).pow(TARGET_DECIMAL.checked_sub(token_decimals).unwrap().into()),
                    )
                    .ok_or(MathError::DivByZero(1))
            } else {
                amount
                    .checked_mul(
                        (10u128).pow(token_decimals.checked_sub(TARGET_DECIMAL).unwrap().into()),
                    )
                    .ok_or(MathError::MulOverflow(1))
            }
        }

        /// Converts provided tokens `amounts` to comparable amounts
        fn to_token_amounts(&self, amounts: &[u128]) -> Result<Vec<u128>, MathError> {
            let mut token_amounts: Vec<u128> = Vec::new();
            for (id, &amount) in amounts.iter().enumerate() {
                token_amounts.push(self.to_token_amount(id, amount)?);
            }
            Ok(token_amounts)
        }

        /// Converts provided comparable `amounts` to tokens amounts
        fn to_comperable_amounts(&self, amounts: &[u128]) -> Result<Vec<u128>, MathError> {
            let mut comperable_amounts: Vec<u128> = Vec::new();
            for (id, &amount) in amounts.iter().enumerate() {
                comperable_amounts.push(self.to_comperable_amount(id, amount)?);
            }
            Ok(comperable_amounts)
        }

        fn ensure_admin(&self) -> Result<(), StablePoolError> {
            ensure!(
                self.env().caller() == self.admin,
                StablePoolError::OnlyAdmin
            );
            Ok(())
        }
    }
    impl StablePool for StablePoolContract {
        #[ink(message)]
        fn tokens(&self) -> Vec<AccountId> {
            self.pool.tokens.clone()
        }

        #[ink(message)]
        fn reserves(&self) -> Vec<u128> {
            self.pool.reserves.clone()
        }

        #[ink(message)]
        fn mint_liquidity(&mut self, to: AccountId) -> Result<u128, StablePoolError> {
            // get transferred amounts (balances sub reserves)
            let mut amounts = Vec::new();
            for (id, &token) in self.pool.tokens.iter().enumerate() {
                amounts.push(
                    self.to_comperable_amount(
                        id,
                        self.token_by_address(token)
                            .balance_of(self.env().account_id()),
                    )?
                    .checked_sub(self.pool.reserves[id])
                    .ok_or(MathError::SubUnderflow(1))?,
                );
            }
            // calc lp tokens (shares_to_mint, fee)
            let (shares, fee_part) = math::compute_lp_amount_for_deposit(
                &amounts,
                &self.pool.reserves,
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef()?,
            )?;
            // mint shares
            let events = self.psp22.mint(to, shares)?;
            self.emit_events(events);
            // mint admin fee
            if let Some(fee_to) = self.fee_to() {
                let events = self
                    .psp22
                    .mint(fee_to, self.pool.fees.admin_trade_fee(fee_part)?)?;
                self.emit_events(events);
            }
            // update reserves
            for (i, &amount) in amounts.iter().enumerate() {
                self.pool.reserves[i] = self.pool.reserves[i]
                    .checked_add(amount)
                    .ok_or(MathError::AddOverflow(2))?;
            }

            Ok(shares)
        }

        #[ink(message)]
        fn burn_liquidity(
            &mut self,
            to: AccountId,
            amounts: Option<Vec<u128>>,
        ) -> Result<(u128, Vec<u128>), StablePoolError> {
            // get transferred amount
            let mut lp_amount = self.psp22.balance_of(self.env().account_id());
            // if amounts is some calc required lp tokens, else no calc w/o fees
            let (amounts_to_withdraw, new_reserves) = if let Some(_amounts) = amounts {
                if _amounts.len() != self.pool.reserves.len() {
                    return Err(StablePoolError::IncorrectAmountsCount);
                }
                let _amounts = self.to_comperable_amounts(&_amounts)?;

                let (lp_to_burn, fee_part) = math::compute_lp_amount_for_withdraw(
                    &_amounts,
                    &self.pool.reserves,
                    self.psp22.total_supply(),
                    Some(&self.pool.fees),
                    self.amp_coef()?,
                )?;
                if lp_to_burn > lp_amount {
                    return Err(StablePoolError::InsufficientLiquidityBurned);
                }
                if let Some(diff) = lp_to_burn.checked_sub(lp_amount) {
                    // transfer difference back to `to`
                    let events = self.psp22.transfer(self.env().account_id(), to, diff)?;
                    self.emit_events(events);
                    // adjust lp_amount (sub returned lp tokens)
                    lp_amount = lp_amount.checked_sub(diff).unwrap(); // it safe because we know that lp_amount > lp_to_burn
                }
                let mut new_reserves = self.pool.reserves.clone();
                for (i, &amount) in _amounts.iter().enumerate() {
                    new_reserves[i] = new_reserves[i]
                        .checked_sub(amount)
                        .ok_or(MathError::SubUnderflow(2))?;
                }
                // mint admin fee
                if let Some(fee_to) = self.fee_to() {
                    let events = self
                        .psp22
                        .mint(fee_to, self.pool.fees.admin_trade_fee(fee_part)?)?;
                    self.emit_events(events);
                }
                (_amounts, new_reserves)
            } else {
                math::compute_withdraw_amounts_for_lp(
                    lp_amount,
                    &self.pool.reserves,
                    self.psp22.total_supply(),
                )?
            };
            // burn shares
            let events = self.psp22.burn(self.env().account_id(), lp_amount)?;
            self.emit_events(events);
            // send tokens to _from
            let amounts_to_withdraw = self.to_token_amounts(&amounts_to_withdraw)?;
            for (id, &amount) in amounts_to_withdraw.iter().enumerate() {
                self.token_by_id(id as u8)?.transfer(
                    to,
                    self.to_comperable_amount(id, amount)?,
                    vec![],
                )?;
            }
            // update reserves
            self.pool.reserves = new_reserves;
            Ok((lp_amount, amounts_to_withdraw))
        }

        #[ink(message)]
        fn swap(
            &mut self,
            token_in_id: u8,
            token_out_id: u8,
            _to: AccountId,
        ) -> Result<(), StablePoolError> {
            //check token ids
            if token_in_id > self.pool.tokens.len() as u8 {
                return Err(StablePoolError::InvalidTokenId(token_in_id));
            }
            if token_out_id > self.pool.tokens.len() as u8 {
                return Err(StablePoolError::InvalidTokenId(token_out_id));
            }
            if token_in_id == token_out_id {
                return Err(StablePoolError::IdenticalTokenId);
            }
            // check amount_in (balance token_in)
            let amount_in = self
                .to_comperable_amount(
                    token_in_id as usize,
                    self.token_by_id(token_in_id)?
                        .balance_of(self.env().account_id()),
                )?
                .checked_sub(self.pool.reserves[token_in_id as usize])
                .ok_or(MathError::SubUnderflow(1))?;
            // get fee_to account
            let fee_to = self.fee_to();
            // calc amount_out and fees
            let swap_res = math::swap_to(
                token_in_id as usize,
                amount_in,
                token_out_id as usize,
                &self.pool.reserves,
                &self.pool.fees,
                self.amp_coef()?,
                fee_to.is_some(),
            )?;
            // transfer token_out
            self.token_by_id(token_out_id)?.transfer(
                _to,
                self.to_token_amount(token_in_id as usize, swap_res.amount_swapped)?,
                vec![],
            )?;
            // update reserves
            self.pool.reserves[token_in_id as usize] = swap_res.new_source_amount;
            self.pool.reserves[token_out_id as usize] = swap_res.new_destination_amount;
            // mint fees for admin
            // "Because amp_coef may change over time, we can't
            // determine the fees only when depositing/withdrawing
            // liquidity, admin fees must be minted and distributed on every swap"
            if fee_to.is_some() && swap_res.admin_fee > 0 {
                let mut admin_deposit_amounts = vec![0u128; self.pool.tokens.len()];
                admin_deposit_amounts[token_out_id as usize] = swap_res.admin_fee;
                // calc shares from
                let (admin_fee_lp, _) = math::compute_lp_amount_for_deposit(
                    &admin_deposit_amounts,
                    &self.pool.reserves,
                    self.psp22.total_supply(),
                    None,
                    self.amp_coef()?,
                )?;
                // mint fee to admin
                let events = self.psp22.mint(fee_to.unwrap(), admin_fee_lp)?;
                self.emit_events(events);
                // update reserve again
                self.pool.reserves[token_out_id as usize] = self.pool.reserves
                    [token_out_id as usize]
                    .checked_add(swap_res.admin_fee)
                    .ok_or(MathError::AddOverflow(1))?;
            }
            Ok(())
        }

        #[ink(message)]
        fn amp_coef(&self) -> Result<u128, StablePoolError> {
            let current_time = self.env().block_timestamp();
            Ok(self.pool.amp_coef.compute_amp_coef(current_time)?)
        }

        #[ink(message)]
        fn ramp_amp_coef(
            &mut self,
            target_amp_coef: u128,
            ramp_duration: u64,
        ) -> Result<(), StablePoolError> {
            self.ensure_admin()?;
            self.pool.amp_coef.ramp_amp_coef(
                target_amp_coef,
                ramp_duration,
                self.env().block_timestamp(),
            )
        }

        #[ink(message)]
        fn set_admin(&mut self, new_admin: AccountId) -> Result<(), StablePoolError> {
            self.ensure_admin()?;
            self.admin = new_admin;
            Ok(())
        }
    }

    impl PSP22 for StablePoolContract {
        #[ink(message)]
        fn total_supply(&self) -> u128 {
            self.psp22.total_supply()
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u128 {
            self.psp22.balance_of(owner)
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.psp22.allowance(owner, spender)
        }

        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let events = self.psp22.transfer(self.env().caller(), to, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let events = self
                .psp22
                .transfer_from(self.env().caller(), from, to, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error> {
            let events = self.psp22.approve(self.env().caller(), spender, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let events =
                self.psp22
                    .increase_allowance(self.env().caller(), spender, delta_value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn decrease_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let events =
                self.psp22
                    .decrease_allowance(self.env().caller(), spender, delta_value)?;
            self.emit_events(events);
            Ok(())
        }
    }

    impl PSP22Metadata for StablePoolContract {
        #[ink(message)]
        fn token_name(&self) -> Option<String> {
            Some("CommonAMM-V2".to_string())
        }

        #[ink(message)]
        fn token_symbol(&self) -> Option<String> {
            Some("CMNAMM-V2".to_string())
        }

        #[ink(message)]
        fn token_decimals(&self) -> u8 {
            12
        }
    }

    #[cfg(test)]
    mod test {
        use ink::primitives::AccountId;

        use super::*;
        #[test]
        fn amount_to_comperable_and_back_1() {
            let stable_swap_contract = StablePoolContract::new(
                vec![],
                vec![6, 6],
                1,
                AccountId::from([0u8; 32]),
                AccountId::from([0u8; 32]),
            );
            let amount: u128 = 1_000_000_000_000; // 1000000.000000
            assert_eq!(
                stable_swap_contract.to_comperable_amount(1, amount),
                Ok(1_000_000_000_000_000_000)
            );
            assert_eq!(
                stable_swap_contract.to_token_amount(1, 1_000_000_000_000_000_000),
                Ok(1_000_000_000_000)
            );
        }

        #[test]
        fn amount_to_comperable_and_back_2() {
            let stable_swap_contract = StablePoolContract::new(
                vec![],
                vec![15, 15],
                1,
                AccountId::from([0u8; 32]),
                AccountId::from([0u8; 32]),
            );
            let amount: u128 = 1_000_000_000_000_000_000_000; // 1000000.000000000000000
            assert_eq!(
                stable_swap_contract.to_comperable_amount(1, amount),
                Ok(1_000_000_000_000_000_000)
            );
            assert_eq!(
                stable_swap_contract.to_token_amount(1, 1_000_000_000_000_000_000),
                Ok(1_000_000_000_000_000_000_000)
            );
        }
    }
}
