#![cfg_attr(not(feature = "std"), no_std, no_main)]
mod token_rate;

#[ink::contract]
pub mod stable_pool {
    use crate::token_rate::TokenRate;
    use amm_helpers::{
        constants::stable_pool::{
            ADMIN_FEE_BPS, MAX_AMP, MIN_AMP, RATE_PRECISION, TOKEN_TARGET_DECIMALS, TRADE_FEE_BPS,
        },
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

    #[ink(event)]
    pub struct OwnerChanged {
        #[ink(topic)]
        pub new_owner: AccountId,
    }

    #[ink(event)]
    pub struct FeeReceiverChanged {
        #[ink(topic)]
        pub new_fee_receiver: Option<AccountId>,
    }

    #[ink(event)]
    pub struct AmpCoefChanged {
        #[ink(topic)]
        pub new_amp_coef: u128,
    }

    #[ink::storage_item]
    #[derive(Debug)]
    pub struct StablePoolData {
        /// List of tokens.
        tokens: Vec<AccountId>,
        /// Tokens precision factors used for normalization.
        precisions: Vec<u128>,
        /// Reserves of tokens
        reserves: Vec<u128>,
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
                tokens_decimals.iter().all(|&d| d <= TOKEN_TARGET_DECIMALS),
                StablePoolError::TooLargeTokenDecimal
            );

            let precisions = tokens_decimals
                .iter()
                .map(|&decimal| {
                    10u128.pow(TOKEN_TARGET_DECIMALS.checked_sub(decimal).unwrap() as u32)
                })
                .collect();
            Ok(Self {
                owner,
                pool: StablePoolData {
                    tokens,
                    reserves: vec![0; token_count],
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
            let token_rates = vec![TokenRate::new_constant(RATE_PRECISION); tokens.len()];
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
                    None => TokenRate::new_constant(RATE_PRECISION),
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

        /// Scaled rates are rates multiplied by precision. They are assumed to fit in u128.
        /// If TOKEN_TARGET_DECIMALS is 18 and RATE_DECIMALS is 12, then rates not exceeding ~340282366 should fit.
        /// That's because if precision <= 10^18 and rate <= 10^12 * 340282366, then rate * precision < 2^128.
        fn get_scaled_rates(&self) -> Result<Vec<u128>, MathError> {
            self.pool
                .token_rates
                .iter()
                .zip(self.pool.precisions.iter())
                .map(|(rate, &precision)| {
                    rate.get_rate()
                        .checked_mul(precision)
                        .ok_or(MathError::MulOverflow(114))
                })
                .collect()
        }

        fn ensure_owner(&self) -> Result<(), StablePoolError> {
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
            if token_in == token_out {
                return Err(StablePoolError::IdenticalTokenId);
            }
            //check token ids
            let token_in_id = self.token_id(token_in)?;
            let token_out_id = self.token_id(token_out)?;
            Ok((token_in_id, token_out_id))
        }

        fn mint_admin_fee(&mut self, fee: u128, token_id: usize) -> Result<(), StablePoolError> {
            if let Some(fee_to) = self.fee_to() {
                let admin_fee = self.pool.fees.admin_trade_fee(fee)?;
                if admin_fee > 0 {
                    let rates = self.get_scaled_rates()?;
                    let mut admin_deposit_amounts = vec![0u128; self.pool.tokens.len()];
                    admin_deposit_amounts[token_id] = admin_fee;
                    let mut reserves = self.pool.reserves.clone();
                    reserves[token_id] = reserves[token_id]
                        .checked_sub(admin_fee)
                        .ok_or(MathError::SubUnderflow(102))?;
                    let (admin_fee_lp, _) = math::rated_compute_lp_amount_for_deposit(
                        &rates,
                        &admin_deposit_amounts,
                        &reserves,
                        self.psp22.total_supply(),
                        None, // no fees
                        self.amp_coef(),
                    )?;
                    // mint fee (shares) to admin
                    let events = self.psp22.mint(fee_to, admin_fee_lp)?;
                    self.emit_events(events);
                }
            }
            Ok(())
        }

        fn decrease_reserve(
            &mut self,
            token_id: usize,
            amount: u128,
        ) -> Result<(), StablePoolError> {
            self.pool.reserves[token_id] = self.pool.reserves[token_id]
                .checked_sub(amount)
                .ok_or(MathError::SubUnderflow(101))?;
            Ok(())
        }

        fn increase_reserve(
            &mut self,
            token_id: usize,
            amount: u128,
        ) -> Result<(), StablePoolError> {
            self.pool.reserves[token_id] = self.pool.reserves[token_id]
                .checked_add(amount)
                .ok_or(MathError::AddOverflow(101))?;
            Ok(())
        }

        /// This method is for internal use only
        /// - calculates token_out amount
        /// - calculates swap fee
        /// - mints admin fee
        /// - updates reserves
        /// It assumes that rates have been updated.
        /// Returns (token_out_amount, swap_fee)
        fn _swap_to(
            &mut self,
            token_in_id: usize,
            token_out_id: usize,
            token_in_amount: u128,
            min_token_out_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            if token_in_amount == 0 {
                return Err(StablePoolError::InsufficientInputAmount);
            }
            let rates = self.get_scaled_rates()?;
            // calc amount_out and fees
            let (token_out_amount, fee) = math::rated_swap_to(
                &rates,
                token_in_id,
                token_in_amount,
                token_out_id,
                &self.reserves(),
                &self.pool.fees,
                self.amp_coef(),
            )?;

            // Check if swapped amount is not less than min_token_out_amount
            if token_out_amount < min_token_out_amount {
                return Err(StablePoolError::InsufficientOutputAmount);
            };
            // update reserves
            self.increase_reserve(token_in_id, token_in_amount)?;
            self.decrease_reserve(token_out_id, token_out_amount)?;

            // mint admin fee
            self.mint_admin_fee(fee, token_out_id)?;
            Ok((token_out_amount, fee))
        }

        /// This method is for internal use only
        /// - calculates token_in amount
        /// - calculates swap fee
        /// - mints admin fee
        /// - updates reserves
        /// It assumes that rates have been updated.
        /// Returns (token_in_amount, swap_fee)
        fn _swap_from(
            &mut self,
            token_in_id: usize,
            token_out_id: usize,
            token_out_amount: u128,
            max_token_in_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            if token_out_amount == 0 {
                return Err(StablePoolError::InsufficientOutputAmount);
            }
            let rates = self.get_scaled_rates()?;
            // calc amount_out and fees
            let (token_in_amount, fee) = math::rated_swap_from(
                &rates,
                token_in_id as usize,
                token_out_amount,
                token_out_id as usize,
                &self.reserves(),
                &self.pool.fees,
                self.amp_coef(),
            )?;

            // Check if in token_in_amount is as constrained by the user
            if token_in_amount > max_token_in_amount {
                return Err(StablePoolError::TooLargeInputAmount);
            };
            // update reserves
            self.increase_reserve(token_in_id, token_in_amount)?;
            self.decrease_reserve(token_out_id, token_out_amount)?;

            // mint admin fee
            self.mint_admin_fee(fee, token_out_id)?;

            // note that fee is applied to token_out (same as in _swap_to)
            Ok((token_in_amount, fee))
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
            // calc lp tokens (shares_to_mint, fee)
            let (shares, fee_part) = self.get_mint_liquidity_for_amounts(amounts.clone())?;

            // Check min shares
            if shares < min_share_amount {
                return Err(StablePoolError::InsufficientLiquidityMinted);
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
                self.increase_reserve(i, amount)?;
            }

            self.env().emit_event(AddLiquidity {
                provider: self.env().caller(),
                token_amounts: amounts,
                shares,
                to,
            });
            Ok((shares, fee_part))
        }

        // Note that this method does not require to update rates, neither it uses rates.
        // Thus it's always possible to call it, even if the rate is outdated, or the rate provider is down.
        #[ink(message)]
        fn remove_liquidity_by_shares(
            &mut self,
            shares: u128,
            min_amounts: Vec<u128>,
            to: AccountId,
        ) -> Result<Vec<u128>, StablePoolError> {
            let amounts = math::compute_amounts_given_lp(
                shares,
                &self.reserves(),
                self.psp22.total_supply(),
            )?;

            // Check if enough tokens are withdrawn
            ensure!(
                amounts
                    .iter()
                    .zip(min_amounts.iter())
                    .all(|(amount, min_amount)| amount >= min_amount),
                StablePoolError::InsufficientOutputAmount
            );

            // transfer tokens
            for (&token, &amount) in self.pool.tokens.iter().zip(amounts.iter()) {
                self.token_by_address(token).transfer(to, amount, vec![])?;
            }

            // update reserves
            for (i, &amount) in amounts.iter().enumerate() {
                self.decrease_reserve(i, amount)?;
            }

            // Burn liquidity
            let events = self.psp22.burn(self.env().caller(), shares)?;
            self.emit_events(events);

            Ok(amounts)
        }

        #[ink(message)]
        fn remove_liquidity_by_amounts(
            &mut self,
            max_share_amount: u128,
            amounts: Vec<u128>,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            // calc comparable amounts
            let (shares_to_burn, fee_part) =
                self.get_burn_liquidity_for_amounts(amounts.clone())?;

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
            for (&token, &amount) in self.pool.tokens.iter().zip(amounts.iter()) {
                self.token_by_address(token).transfer(to, amount, vec![])?;
            }
            // update reserves
            for (i, &amount) in amounts.iter().enumerate() {
                self.decrease_reserve(i, amount)?;
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
        fn force_update_rate(&mut self) {
            let current_time = self.env().block_timestamp();
            for rate in self.pool.token_rates.iter_mut() {
                rate.update_rate_no_cache(current_time);
            }
        }

        #[ink(message)]
        fn swap_exact_in(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_in_amount: u128,
            min_token_out_amount: u128,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            //check token ids
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;

            // Make sure rates are up to date before we attempt any calculations
            self.update_rates();

            // transfer token_in
            self.token_by_address(token_in).transfer_from(
                self.env().caller(),
                self.env().account_id(),
                token_in_amount,
                vec![],
            )?;

            // calculate amount out, mint admin fee and update reserves
            let (token_out_amount, swap_fee) = self._swap_to(
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
        fn swap_exact_out(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_out_amount: u128,
            max_token_in_amount: u128,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            //check token ids
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;

            // Make sure rates are up to date before we attempt any calculations
            self.update_rates();

            // calculate amount out, mint admin fee and update reserves
            let (token_in_amount, swap_fee) = self._swap_from(
                token_in_id,
                token_out_id,
                token_out_amount,
                max_token_in_amount,
            )?;

            // transfer token_in
            self.token_by_address(token_in).transfer_from(
                self.env().caller(),
                self.env().account_id(),
                token_in_amount,
                vec![],
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
            Ok((token_in_amount, swap_fee))
        }

        #[ink(message)]
        fn set_owner(&mut self, new_owner: AccountId) -> Result<(), StablePoolError> {
            self.ensure_owner()?;
            self.owner = new_owner;
            self.env().emit_event(OwnerChanged { new_owner });
            Ok(())
        }

        #[ink(message)]
        fn set_fee_receiver(
            &mut self,
            fee_receiver: Option<AccountId>,
        ) -> Result<(), StablePoolError> {
            self.ensure_owner()?;
            self.pool.fee_receiver = fee_receiver;
            self.env().emit_event(FeeReceiverChanged {
                new_fee_receiver: fee_receiver,
            });
            Ok(())
        }

        #[ink(message)]
        fn set_amp_coef(&mut self, amp_coef: u128) -> Result<(), StablePoolError> {
            self.ensure_owner()?;
            validate_amp_coef(amp_coef)?;
            self.pool.amp_coef = amp_coef;
            self.env().emit_event(AmpCoefChanged {
                new_amp_coef: amp_coef,
            });
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
            self.pool.reserves.clone()
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
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;
            self.update_rates();
            let rates = self.get_scaled_rates()?;
            Ok(math::rated_swap_to(
                &rates,
                token_in_id as usize,
                token_in_amount,
                token_out_id as usize,
                &self.reserves(),
                &self.pool.fees,
                self.amp_coef(),
            )?)
        }

        #[ink(message)]
        fn get_swap_amount_in(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_out_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;
            self.update_rates();
            let rates = self.get_scaled_rates()?;
            Ok(math::rated_swap_from(
                &rates,
                token_in_id as usize,
                token_out_amount,
                token_out_id as usize,
                &self.reserves(),
                &self.pool.fees,
                self.amp_coef(),
            )?)
        }

        #[ink(message)]
        fn get_mint_liquidity_for_amounts(
            &mut self,
            amounts: Vec<u128>,
        ) -> Result<(u128, u128), StablePoolError> {
            if amounts.len() != self.pool.tokens.len() {
                return Err(StablePoolError::IncorrectAmountsCount);
            }
            self.update_rates();
            let rates = self.get_scaled_rates()?;

            Ok(math::rated_compute_lp_amount_for_deposit(
                &rates,
                &amounts,
                &self.reserves(),
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef(),
            )?)
        }

        #[ink(message)]
        fn get_amounts_for_liquidity_mint(
            &mut self,
            liquidity: u128,
        ) -> Result<Vec<u128>, StablePoolError> {
            match math::compute_amounts_given_lp(
                liquidity,
                &self.reserves(),
                self.psp22.total_supply(),
            ) {
                Ok(amounts) => Ok(amounts),
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
            let rates = self.get_scaled_rates()?;
            math::rated_compute_lp_amount_for_withdraw(
                &rates,
                &amounts,
                &self.reserves(),
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
            match math::compute_amounts_given_lp(
                liquidity,
                &self.reserves(),
                self.psp22.total_supply(),
            ) {
                Ok(amounts) => Ok(amounts),
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
            TOKEN_TARGET_DECIMALS
        }
    }
}
