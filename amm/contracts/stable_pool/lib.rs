#![cfg_attr(not(feature = "std"), no_std, no_main)]
mod token_rate;

#[ink::contract]
pub mod stable_pool {
    // amount * 0.06%
    pub const TRADE_FEE_BPS: u32 = 6;
    // amount * 0.06% * 20% (part of the TRADE_FEE)
    pub const ADMIN_FEE_BPS: u32 = 2_000;

    pub const TARGET_DECIMALS: u8 = 24;
    pub const TARGET_PRECISION: u128 = 10u128.pow(TARGET_DECIMALS as u32);

    /// Min amplification coefficient.
    pub const MIN_AMP: u128 = 1;
    /// Max amplification coefficient.
    pub const MAX_AMP: u128 = 1_000_000;

    use crate::token_rate::TokenRate;
    use amm_helpers::{
        ensure,
        stable_swap_math::{self as math, fees::Fees},
    };
    use ink::contract_ref;
    use ink::prelude::{
        string::{String, ToString},
        {vec, vec::Vec},
    };
    use psp22::{PSP22Data, PSP22Error, PSP22Event, PSP22Metadata, PSP22};
    use traits::{MathError, StablePool, StablePoolError, StablePoolView};

    #[ink(event)]
    pub struct AddLiquidity {
        #[ink(topic)]
        pub provider: AccountId,
        pub token_amounts: Vec<u128>,
        pub shares: u128,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct RemoveLiquidity {
        #[ink(topic)]
        pub provider: AccountId,
        pub token_amounts: Vec<u128>,
        pub shares: u128,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct Swap {
        #[ink(topic)]
        pub sender: AccountId,
        pub token_in: AccountId,
        pub amount_in: u128,
        pub token_out: AccountId,
        pub amount_out: u128,
        #[ink(topic)]
        pub to: AccountId,
    }
    #[ink(event)]
    pub struct RampAmpCoef {
        pub old_amp_coef: u128,
        pub new_amp_coef: u128,
        pub init_time: u64,
        pub ramp_duration: u64,
    }

    #[ink(event)]
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
        /// List of tokens.
        tokens: Vec<AccountId>,
        /// Tokens precision factors used for normalization.
        precisions: Vec<u128>,
        /// Reserves in comparable amounts.
        c_reserves: Vec<u128>,
        /// Means of getting token rates, either constant or external contract call.
        token_rates: Vec<TokenRate>,
        /// Amplification coefficient.
        amp_coef: u128,
        /// Fees
        fees: Fees,
        /// Who receives admin fees (if any).
        fee_receiver: Option<AccountId>,
    }

    #[ink(storage)]
    pub struct StablePoolContract {
        owner: AccountId,
        pool: StablePoolData,
        psp22: PSP22Data,
    }

    fn validate_amp_coef(amp_coef: u128) -> Result<(), StablePoolError> {
        ensure!(
            amp_coef >= MIN_AMP && amp_coef <= MAX_AMP,
            StablePoolError::InvalidAmpCoef
        );
        Ok(())
    }

    impl StablePoolContract {
        pub fn new_pool(
            tokens: Vec<AccountId>,
            tokens_decimals: Vec<u8>,
            token_rates: Vec<TokenRate>,
            amp_coef: u128,
            owner: AccountId,
            fee_receiver: Option<AccountId>,
        ) -> Result<Self, StablePoolError> {
            validate_amp_coef(amp_coef)?;
            let mut unique_tokens = tokens.clone();
            unique_tokens.sort();
            unique_tokens.dedup();
            let token_count = tokens.len();
            ensure!(
                unique_tokens.len() == token_count,
                StablePoolError::IdenticalTokenId
            );
            ensure!(
                token_count == tokens_decimals.len()
                    && token_count == token_rates.len()
                    && token_count > 1,
                StablePoolError::IncorrectTokenCount
            );

            ensure!(
                tokens_decimals.iter().all(|&d| d <= TARGET_DECIMALS),
                StablePoolError::TooLargeTokenDecimal
            );

            let precisions = tokens_decimals
                .iter()
                .map(|&decimal| 10u128.pow(TARGET_DECIMALS.checked_sub(decimal).unwrap() as u32))
                .collect();
            Ok(Self {
                owner,
                pool: StablePoolData {
                    tokens,
                    c_reserves: vec![0; token_count],
                    precisions,
                    token_rates,
                    amp_coef,
                    fees: Fees::new(TRADE_FEE_BPS, ADMIN_FEE_BPS),
                    fee_receiver,
                },
                psp22: PSP22Data::default(),
            })
        }

        #[ink(constructor)]
        pub fn new_stable(
            tokens: Vec<AccountId>,
            tokens_decimals: Vec<u8>,
            init_amp_coef: u128,
            owner: AccountId,
            fee_receiver: Option<AccountId>,
        ) -> Result<Self, StablePoolError> {
            let token_rates =
                vec![TokenRate::new_constant(10u128.pow(TARGET_DECIMALS as u32)); tokens.len()];
            Self::new_pool(
                tokens,
                tokens_decimals,
                token_rates,
                init_amp_coef,
                owner,
                fee_receiver,
            )
        }

        #[ink(constructor)]
        pub fn new_rated(
            tokens: Vec<AccountId>,
            tokens_decimals: Vec<u8>,
            external_rates: Vec<Option<AccountId>>,
            rate_expiration_duration_ms: u64,
            init_amp_coef: u128,
            owner: AccountId,
            fee_receiver: Option<AccountId>,
        ) -> Result<Self, StablePoolError> {
            let current_time = Self::env().block_timestamp();
            let token_rates = external_rates
                .into_iter()
                .map(|rate| match rate {
                    Some(contract) => {
                        TokenRate::new_external(current_time, contract, rate_expiration_duration_ms)
                    }
                    None => TokenRate::new_constant(10u128.pow(TARGET_DECIMALS as u32)),
                })
                .collect();
            Self::new_pool(
                tokens,
                tokens_decimals,
                token_rates,
                init_amp_coef,
                owner,
                fee_receiver,
            )
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

        #[inline]
        fn token_by_address(&self, address: AccountId) -> contract_ref!(PSP22) {
            address.into()
        }

        fn update_rates(&mut self) {
            let current_time = self.env().block_timestamp();
            for rate in self.pool.token_rates.iter_mut() {
                rate.update_rate(current_time);
            }
        }

        fn fee_to(&self) -> Option<AccountId> {
            self.pool.fee_receiver
        }

        /// Converts provided token `amount` to comparable amount
        fn amount_to_c_amount(&self, amount: u128, token_id: usize) -> Result<u128, MathError> {
            amount
                .checked_mul(self.pool.precisions[token_id])
                .ok_or(MathError::MulOverflow(101))
        }

        fn c_amount_to_r_amount(&self, c_amount: u128, token_id: usize) -> Result<u128, MathError> {
            let rate = self.pool.token_rates[token_id].get_rate();
            // Little gas optimization for the most common case
            if rate == TARGET_PRECISION {
                return Ok(c_amount);
            }

            Ok(c_amount
                .checked_mul(rate)
                .ok_or(MathError::MulOverflow(102))?
                // Unwrap is safe since TARGET_PRECISION > 0
                .checked_div(TARGET_PRECISION).unwrap())
        }

        fn amount_to_r_amount(&self, amount: u128, token_id: usize) -> Result<u128, MathError> {
            self.c_amount_to_r_amount(self.amount_to_c_amount(amount, token_id)?, token_id)
        }

        /// Converts provided comparable `amount` to token amount
        fn c_amount_to_amount(&self, amount: u128, token_id: usize) -> u128 {
            // it is safe to unwrap since precision for any token is >= 1
            amount.checked_div(self.pool.precisions[token_id]).unwrap()
        }

        fn r_amount_to_c_amount(&self, r_amount: u128, token_id: usize) -> Result<u128, MathError> {
            let rate = self.pool.token_rates[token_id].get_rate();
            // Little gas optimization for the most common case
            if rate == TARGET_PRECISION {
                return Ok(r_amount);
            }
            Ok(r_amount
                .checked_mul(TARGET_PRECISION)
                .ok_or(MathError::MulOverflow(103))?
                .checked_div(rate)
                .ok_or(MathError::DivByZero(103))?)
        }

        fn r_amount_to_amount(&self, r_amount: u128, token_id: usize) -> Result<u128, MathError> {
            Ok(self.c_amount_to_amount(self.r_amount_to_c_amount(r_amount, token_id)?, token_id))
        }

        /// Converts provided tokens `amounts` to comparable amounts
        fn c_amounts_to_amounts(&self, amounts: &[u128]) -> Vec<u128> {
            amounts
                .iter()
                .enumerate()
                .map(|(id, &amount)| self.c_amount_to_amount(amount, id))
                .collect()
        }

        fn amounts_to_r_amounts(&self, amounts: &[u128]) -> Result<Vec<u128>, MathError> {
            amounts
                .iter()
                .enumerate()
                .map(|(id, &amount)| self.amount_to_r_amount(amount, id))
                .collect()
        }

        fn r_amounts_to_amounts(&self, r_amounts: &[u128]) -> Vec<u128> {
            r_amounts
                .iter()
                .enumerate()
                .map(|(id, &r_amount)| self.r_amount_to_amount(r_amount, id).unwrap())
                .collect()
        }

        fn ensure_onwer(&self) -> Result<(), StablePoolError> {
            ensure!(
                self.env().caller() == self.owner,
                StablePoolError::OnlyOwner
            );
            Ok(())
        }

        fn token_id(&self, token: AccountId) -> Result<usize, StablePoolError> {
            self.pool
                .tokens
                .iter()
                .position(|&id| id == token)
                .ok_or(StablePoolError::InvalidTokenId(token))
        }

        /// Checks if tokens are valid and returns the tokens ids
        fn check_tokens(
            &self,
            token_in: AccountId,
            token_out: AccountId,
        ) -> Result<(usize, usize), StablePoolError> {
            //check token ids
            let token_in_id = self.token_id(token_in)?;
            let token_out_id = self.token_id(token_out)?;
            if token_in_id == token_out_id {
                return Err(StablePoolError::IdenticalTokenId);
            }
            Ok((token_in_id, token_out_id))
        }

        fn get_r_reserves(&self) -> Result<Vec<u128>, MathError> {
            self.pool
                .c_reserves
                .iter()
                .enumerate()
                .map(|(id, &c_amount)| self.c_amount_to_r_amount(c_amount, id))
                .collect()
        }

        /// This method is for internal use only
        /// - calculates token_out amount
        /// - calculates swap fee
        /// - mints admin fee
        /// - updates reserves
        /// It assumes that rates have been updated.
        /// Returns (token_out_amount, swap_fee)
        fn _swap(
            &mut self,
            token_in_id: usize,
            token_out_id: usize,
            token_in_amount: u128,
            min_token_out_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            if token_in_amount == 0 {
                return Err(StablePoolError::InsufficientInputAmount);
            }
            // convert token_in_amount to comparable amount
            let token_in_c_amount = self.amount_to_c_amount(token_in_amount, token_in_id)?;
            let token_in_r_amount = self.c_amount_to_r_amount(token_in_c_amount, token_in_id)?;
            // calc amount_out and fees
            let (token_out_r_amount, r_fee) = math::swap_to(
                token_in_id,
                token_in_r_amount,
                token_out_id,
                &self.get_r_reserves()?,
                &self.pool.fees,
                self.amp_coef(),
            )?;
            let token_out_c_amount = self.r_amount_to_c_amount(token_out_r_amount, token_out_id)?;
            let token_out_amount = self.c_amount_to_amount(token_out_c_amount, token_out_id);
            // Check if swapped amount is not less than min_token_out_amount
            if token_out_amount < min_token_out_amount {
                return Err(StablePoolError::InsufficientOutputAmount);
            };
            // update reserves
            self.pool.c_reserves[token_in_id] = self.pool.c_reserves[token_in_id]
                .checked_add(token_in_c_amount)
                .ok_or(MathError::AddOverflow(101))?;
            self.pool.c_reserves[token_out_id] = self.pool.c_reserves[token_out_id]
                .checked_sub(token_out_c_amount)
                .ok_or(MathError::SubUnderflow(101))?;

            // distribute admin fee
            if let Some(fee_to) = self.fee_to() {
                let r_admin_fee = self.pool.fees.admin_trade_fee(r_fee)?;
                if r_admin_fee > 0 {
                    let mut r_admin_deposit_amounts = vec![0u128; self.pool.tokens.len()];
                    r_admin_deposit_amounts[token_out_id] = r_admin_fee;
                    let mut r_reserves = self.get_r_reserves()?;
                    r_reserves[token_out_id] = r_reserves[token_out_id]
                        .checked_sub(r_admin_fee)
                        .ok_or(MathError::SubUnderflow(102))?;
                    let (admin_fee_lp, _) = math::compute_lp_amount_for_deposit(
                        &r_admin_deposit_amounts,
                        &r_reserves,
                        self.psp22.total_supply(),
                        None, // no fees
                        self.amp_coef(),
                    )?;
                    // mint fee (shares) to admin
                    let events = self.psp22.mint(fee_to, admin_fee_lp)?;
                    self.emit_events(events);
                }
            }
            Ok((
                token_out_amount,
                self.r_amount_to_amount(r_fee, token_out_id)?,
            ))
        }
    }

    impl StablePool for StablePoolContract {
        #[ink(message)]
        fn add_liquidity(
            &mut self,
            min_share_amount: u128,
            amounts: Vec<u128>,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            // Make sure rates are up to date before we attempt any calculations
            self.update_rates();
            if amounts.len() != self.pool.tokens.len() {
                return Err(StablePoolError::IncorrectAmountsCount);
            }

            // transfer amounts
            for (id, &token) in self.pool.tokens.iter().enumerate() {
                self.token_by_address(token).transfer_from(
                    self.env().caller(),
                    self.env().account_id(),
                    amounts[id],
                    vec![],
                )?;
            }

            let r_amounts: Vec<u128> = amounts
                .iter()
                .enumerate()
                .map(|(id, &amount)| self.amount_to_r_amount(amount, id))
                .collect::<Result<Vec<u128>, MathError>>()?;

            // calc lp tokens (shares_to_mint, fee)
            let (shares, fee_part) = math::compute_lp_amount_for_deposit(
                &r_amounts,
                &self.get_r_reserves()?,
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef(),
            )?;

            // Check min shares
            if shares < min_share_amount {
                return Err(StablePoolError::InsufficientLiquidityMinted);
            }

            // mint shares
            let events = self.psp22.mint(to, shares)?;
            self.emit_events(events);

            // mint admin fee
            if let Some(fee_to) = self.fee_to() {
                let admin_fee = self.pool.fees.admin_trade_fee(fee_part)?;
                if admin_fee > 0 {
                    let events = self.psp22.mint(fee_to, admin_fee)?;
                    self.emit_events(events);
                }
            }

            // update reserves
            for (i, &amount) in amounts.iter().enumerate() {
                self.pool.c_reserves[i] = self.pool.c_reserves[i]
                    .checked_add(self.amount_to_c_amount(amount, i)?)
                    .ok_or(MathError::AddOverflow(101))?;
            }

            self.env().emit_event(AddLiquidity {
                provider: self.env().caller(),
                token_amounts: amounts,
                shares,
                to,
            });
            Ok((shares, fee_part))
        }

        #[ink(message)]
        fn remove_liquidity(
            &mut self,
            max_share_amount: u128,
            amounts: Vec<u128>,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            // Make sure rates are up to date before we attempt any calculations
            self.update_rates();
            if amounts.len() != self.pool.tokens.len() {
                return Err(StablePoolError::IncorrectAmountsCount);
            }

            // calc comparable amounts
            let r_amounts = self.amounts_to_r_amounts(&amounts)?;
            let (shares_to_burn, fee_part) = math::compute_lp_amount_for_withdraw(
                &r_amounts,
                &self.get_r_reserves()?,
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef(),
            )?;
            // check max shares
            if shares_to_burn > max_share_amount {
                return Err(StablePoolError::InsufficientLiquidityBurned);
            }
            // burn shares
            let events = self.psp22.burn(self.env().caller(), shares_to_burn)?;
            self.emit_events(events);
            // mint admin fee
            if let Some(fee_to) = self.fee_to() {
                let admin_fee = self.pool.fees.admin_trade_fee(fee_part)?;
                if admin_fee > 0 {
                    let events = self.psp22.mint(fee_to, admin_fee)?;
                    self.emit_events(events);
                }
            }
            // transfer tokens
            for (id, &token) in self.pool.tokens.iter().enumerate() {
                self.token_by_address(token)
                    .transfer(to, amounts[id], vec![])?;
            }
            // update reserves
            for (i, &amount) in amounts.iter().enumerate() {
                self.pool.c_reserves[i] = self.pool.c_reserves[i]
                    .checked_sub(self.amount_to_c_amount(amount, i)?)
                    .ok_or(MathError::SubUnderflow(101))?;
            }
            self.env().emit_event(RemoveLiquidity {
                provider: self.env().caller(),
                token_amounts: amounts,
                shares: shares_to_burn,
                to,
            });
            Ok((shares_to_burn, fee_part))
        }


        #[ink(message)]
        fn force_update_rate(
            &mut self,
        ) {
            let current_time = self.env().block_timestamp();
            for rate in self.pool.token_rates.iter_mut() {
                rate.update_rate_no_cache(current_time);
            }
        }

        #[ink(message)]
        fn swap(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_in_amount: u128,
            min_token_out_amount: u128,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            // Make sure rates are up to date before we attempt any calculations
            self.update_rates();

            //check token ids
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;

            // transfer token_in
            self.token_by_address(token_in).transfer_from(
                self.env().caller(),
                self.env().account_id(),
                token_in_amount,
                vec![],
            )?;

            // calculate amount out, mint admin fee and update reserves
            let (token_out_amount, swap_fee) = self._swap(
                token_in_id,
                token_out_id,
                token_in_amount,
                min_token_out_amount,
            )?;

            // transfer token_out
            self.token_by_address(token_out)
                .transfer(to, token_out_amount, vec![])?;

            self.env().emit_event(Swap {
                sender: self.env().caller(),
                token_in,
                amount_in: token_in_amount,
                token_out,
                amount_out: token_out_amount,
                to,
            });
            Ok((token_out_amount, swap_fee))
        }

        #[ink(message)]
        fn set_owner(&mut self, new_owner: AccountId) -> Result<(), StablePoolError> {
            self.ensure_onwer()?;
            self.owner = new_owner;
            Ok(())
        }

        #[ink(message)]
        fn set_fee_receiver(
            &mut self,
            fee_receiver: Option<AccountId>,
        ) -> Result<(), StablePoolError> {
            self.ensure_onwer()?;
            self.pool.fee_receiver = fee_receiver;
            Ok(())
        }

        #[ink(message)]
        fn set_amp_coef(
            &mut self,
            amp_coef: u128,
        ) -> Result<(), StablePoolError> {
            self.ensure_onwer()?;
            validate_amp_coef(amp_coef)?;
            self.pool.amp_coef = amp_coef;
            Ok(())
        }
    }

    impl StablePoolView for StablePoolContract {
        #[ink(message)]
        fn tokens(&self) -> Vec<AccountId> {
            self.pool.tokens.clone()
        }

        // This can output values lower than the actual balances of these tokens, which stems from roundings.
        // However an invariant holds that each balance is at least the value returned by this function.
        #[ink(message)]
        fn reserves(&self) -> Vec<u128> {
            self.c_amounts_to_amounts(&self.pool.c_reserves)
        }

        #[ink(message)]
        fn amp_coef(&self) -> u128 {
            self.pool.amp_coef
        }

        #[ink(message)]
        fn get_swap_amount_out(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_in_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            self.update_rates();
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;
            let (r_token_out_amount, r_fee) = math::swap_to(
                token_in_id as usize,
                self.amount_to_r_amount(token_in_amount, token_in_id)?,
                token_out_id as usize,
                &self.get_r_reserves()?,
                &self.pool.fees,
                self.amp_coef(),
            )?;
            Ok((
                self.r_amount_to_amount(r_token_out_amount, token_out_id)?,
                self.r_amount_to_amount(r_fee, token_out_id)?,
            ))
        }

        #[ink(message)]
        fn get_swap_amount_in(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_out_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            self.update_rates();
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;
            let (r_token_in_amount, r_fee) = math::swap_from(
                token_in_id as usize,
                self.amount_to_r_amount(token_out_amount, token_out_id)?,
                token_out_id as usize,
                &self.get_r_reserves()?,
                &self.pool.fees,
                self.amp_coef(),
            )?;
            Ok((
                self.r_amount_to_amount(r_token_in_amount, token_in_id)?,
                self.r_amount_to_amount(r_fee, token_out_id)?,
            ))
        }

        #[ink(message)]
        fn get_mint_liquidity_for_amounts(
            &mut self,
            amounts: Vec<u128>,
        ) -> Result<(u128, u128), StablePoolError> {
            self.update_rates();
            if amounts.len() != self.pool.tokens.len() {
                return Err(StablePoolError::IncorrectAmountsCount);
            }
            math::compute_lp_amount_for_deposit(
                &self.amounts_to_r_amounts(&amounts)?,
                &self.get_r_reserves()?,
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef(),
            )
            .map_err(|err| StablePoolError::MathError(err))
        }

        #[ink(message)]
        fn get_amounts_for_liquidity_mint(
            &mut self,
            liquidity: u128,
        ) -> Result<Vec<u128>, StablePoolError> {
            self.update_rates();
            match math::compute_deposit_amounts_for_lp(
                liquidity,
                &self.get_r_reserves()?,
                self.psp22.total_supply(),
            ) {
                Ok((amounts, _)) => Ok(self.r_amounts_to_amounts(&amounts)),
                Err(err) => Err(StablePoolError::MathError(err)),
            }
        }

        #[ink(message)]
        fn get_burn_liquidity_for_amounts(
            &mut self,
            amounts: Vec<u128>,
        ) -> Result<(u128, u128), StablePoolError> {
            self.update_rates();
            if amounts.len() != self.pool.tokens.len() {
                return Err(StablePoolError::IncorrectAmountsCount);
            }
            math::compute_lp_amount_for_withdraw(
                &self.amounts_to_r_amounts(&amounts)?,
                &self.get_r_reserves()?,
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef(),
            )
            .map_err(|err| StablePoolError::MathError(err))
        }

        #[ink(message)]
        fn get_amounts_for_liquidity_burn(
            &mut self,
            liquidity: u128,
        ) -> Result<Vec<u128>, StablePoolError> {
            self.update_rates();
            match math::compute_withdraw_amounts_for_lp(
                liquidity,
                &self.get_r_reserves()?,
                self.psp22.total_supply(),
            ) {
                Ok((amounts, _)) => Ok(self.r_amounts_to_amounts(&amounts)),
                Err(err) => Err(StablePoolError::MathError(err)),
            }
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
            Some("CommonStableSwap".to_string())
        }

        #[ink(message)]
        fn token_symbol(&self) -> Option<String> {
            Some("CMNSS".to_string())
        }

        #[ink(message)]
        fn token_decimals(&self) -> u8 {
            TARGET_DECIMALS
        }
    }

    #[cfg(test)]
    mod test {
        use ink::primitives::AccountId;

        use super::*;
        #[test]
        fn amount_to_comparable_and_back_1() {
            let stable_pool_contract = StablePoolContract::new_stable(
                vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])],
                vec![6, 12],
                1,
                AccountId::from([0u8; 32]),
                None,
            )
            .map_err(|err| panic!("Contract instantiation error: {err:?}"))
            .unwrap();
            let amount: u128 = 1_000_000_000_000; // 1000000.000000
            let expect_amount: u128 = amount * 10u128.pow((TARGET_DECIMALS - 6) as u32); // 1000000.000000000000000000
            assert_eq!(
                stable_pool_contract.amount_to_r_amount(amount, 0),
                Ok(expect_amount)
            );
            assert_eq!(
                stable_pool_contract
                    .r_amount_to_amount(expect_amount, 0)
                    .unwrap(),
                amount
            );
            let amount: u128 = 1_000_000_000_000_000_000; // 1000000.000000000000
            let expect_amount: u128 = amount * 10u128.pow((TARGET_DECIMALS - 12) as u32);
            assert_eq!(
                stable_pool_contract.amount_to_r_amount(amount, 1),
                Ok(expect_amount)
            );
            assert_eq!(
                stable_pool_contract
                    .r_amount_to_amount(expect_amount, 1)
                    .unwrap(),
                amount
            );
        }

        #[test]
        fn amount_to_comparable_and_back_2() {
            let stable_pool_contract = StablePoolContract::new_stable(
                vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])],
                vec![0, 24],
                1,
                AccountId::from([0u8; 32]),
                Some(AccountId::from([0u8; 32])),
            )
            .map_err(|err| panic!("Contract instantiation error: {err:?}"))
            .unwrap();
            let amount: u128 = 1_000_000; // 1000000
            let expect_amount: u128 = amount * 10u128.pow(TARGET_DECIMALS as u32); // 1000000.000000000000000000000000
            assert_eq!(
                stable_pool_contract.amount_to_r_amount(amount, 0),
                Ok(expect_amount)
            );
            assert_eq!(
                stable_pool_contract.r_amount_to_amount(expect_amount, 0),
                Ok(amount)
            );
            let amount: u128 = 1_000_000_000_000_000_000_000_000_000_000; // 1000000.000000000000000000000000
            let expect_amount: u128 = amount;
            assert_eq!(
                stable_pool_contract.amount_to_r_amount(amount, 1),
                Ok(expect_amount)
            );
            assert_eq!(
                stable_pool_contract.r_amount_to_amount(expect_amount, 1),
                Ok(amount)
            );
        }
    }
}
