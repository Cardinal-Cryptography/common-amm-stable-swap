#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod amp_coef;
pub mod fees;
pub mod math;

#[ink::contract]
pub mod sazero_azero_pair {
    // 0.0006 * amount
    pub const TRADE_FEE_BPS: u32 = 6;
    // 0.0006 * 0.2 * amount (part of the TRADE_FEE)
    pub const ADMIN_FEE_BPS: u32 = 2_000;
    // 24 h
    pub const RATE_EXPIRE_TS: u64 = 86400000;

    pub const SAZERO_DECIMALS: u8 = 18;
    pub const AZERO_DECIMALS: u8 = 12;
    pub const ONE_SAZERO: u128 = 10u128.pow(SAZERO_DECIMALS as u32);
    pub const ONE_AZERO: u128 = 10u128.pow(AZERO_DECIMALS as u32);

    pub use crate::amp_coef::*;
    pub use crate::fees::*;
    pub use crate::math;
    use amm_helpers::ensure;
    use amm_helpers::math::casted_mul;
    use ink::contract_ref;
    use ink::prelude::{
        string::{String, ToString},
        {vec, vec::Vec},
    };
    use psp22::{PSP22Data, PSP22Error, PSP22Event, PSP22Metadata, PSP22};
    use sazero_rate_mock_contract::SazeroRate as SazeroRateContractTrait;
    use traits::{Factory, MathError, StablePool, StablePoolError, StablePoolView};

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
    pub struct SazeroAzeroPairData {
        /// Factory contract
        factory: contract_ref!(Factory),
        /// Tokens.
        tokens: Vec<AccountId>,
        /// Tokens precision factors used for normalization.
        precisions: Vec<u128>,
        /// Reserves in comparable amounts.
        reserves: Vec<u128>,
        /// Amplification coefficient.
        amp_coef: AmplificationCoefficient,
        /// Fees
        fees: Fees,
        ///
        cached_sazero_rate: u128,
        ///
        last_sazero_rate_update_ts: u64,
        ///
        sazero_rate_contract: contract_ref!(SazeroRateContractTrait),
    }

    #[ink(storage)]
    pub struct SazeroAzeroPairContract {
        owner: AccountId,
        pool: SazeroAzeroPairData,
        psp22: PSP22Data,
        decimals: u8,
    }

    impl SazeroAzeroPairContract {
        #[ink(constructor)]
        pub fn new(
            sazero: AccountId,
            wazero: AccountId,
            sazero_rate_contract: AccountId,
            init_amp_coef: u128,
            factory: AccountId,
            owner: AccountId,
        ) -> Self {
            let reserves = vec![0, 0];
            let precisions = vec![
                1,
                10u128.pow(SAZERO_DECIMALS.checked_sub(AZERO_DECIMALS).unwrap().into()),
            ];
            let mut instance = Self {
                owner,
                pool: SazeroAzeroPairData {
                    factory: factory.into(),
                    tokens: vec![sazero, wazero],
                    precisions,
                    reserves,
                    amp_coef: AmplificationCoefficient::new(init_amp_coef),
                    fees: Fees::new(TRADE_FEE_BPS, ADMIN_FEE_BPS),
                    cached_sazero_rate: ONE_AZERO,
                    last_sazero_rate_update_ts: 0,
                    sazero_rate_contract: sazero_rate_contract.into(),
                },
                psp22: PSP22Data::default(),
                decimals: SAZERO_DECIMALS,
            };
            instance.update_sazero_rate();
            instance
        }

        // TODO: probably to remove
        #[ink(constructor)]
        pub fn new_test(
            sazero: AccountId,
            wazero: AccountId,
            sazero_rate_contract: AccountId,
            init_amp_coef: u128,
            factory: AccountId,
            owner: AccountId,
        ) -> Result<Self, StablePoolError> {
            ensure!(sazero != wazero, StablePoolError::IdenticalTokenId);
            ensure!(init_amp_coef >= MIN_AMP, StablePoolError::AmpCoefTooLow);
            ensure!(init_amp_coef <= MAX_AMP, StablePoolError::AmpCoefTooHigh);
            Ok(Self::new(
                sazero,
                wazero,
                sazero_rate_contract,
                init_amp_coef,
                factory,
                owner,
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
        fn token_by_address(&self, address: AccountId) -> contract_ref!(PSP22) {
            address.into()
        }

        fn fee_to(&self) -> Option<AccountId> {
            self.pool.factory.fee_to()
        }

        /// Converts provided token `amount` to comparable amount
        fn to_comperable_amount(&self, token_id: usize, amount: u128) -> Result<u128, MathError> {
            amount
                .checked_mul(self.pool.precisions[token_id])
                .ok_or(MathError::MulOverflow(1))
        }

        /// Converts provided comparable `amount` to token amount
        fn to_token_amount(&self, token_id: usize, amount: u128) -> Result<u128, MathError> {
            amount
                .checked_div(self.pool.precisions[token_id])
                .ok_or(MathError::DivByZero(1))
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

        fn ensure_onwer(&self) -> Result<(), StablePoolError> {
            ensure!(
                self.env().caller() == self.owner,
                StablePoolError::OnlyAdmin
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

        /// This method is for internal use only
        /// - calculates token_out amount
        /// - calculates swap fee
        /// - mints admin fee
        /// - updates reserves
        /// Returns (token_out_amount, swap_fee)
        fn _swap(
            &mut self,
            token_in_id: usize,
            token_out_id: usize,
            comparable_token_in_amount: u128,
            min_token_out_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            if comparable_token_in_amount == 0 {
                return Err(StablePoolError::InsufficientInputAmount);
            }
            // get fee_to account
            let fee_to = self.fee_to();
            // rate reserve and amount in
            let rate = self.get_actual_sazero_rate_and_update();
            let rated_comparable_token_in_amount =
                Self::amount_to_rated_amount(comparable_token_in_amount, rate, token_in_id)?;
            // calc amount_out and fees
            let swap_res = math::swap_to(
                token_in_id,
                rated_comparable_token_in_amount,
                token_out_id,
                &self.rated_reserves(rate)?,
                &self.pool.fees,
                self.amp_coef()?,
                fee_to.is_some(),
            )?;
            let amount_swapped =
                Self::rated_amount_to_amount(swap_res.amount_swapped, rate, token_out_id)?;
            let token_out_amount = self.to_token_amount(token_out_id, amount_swapped)?;
            // Check if swapped amount is not less than min_token_out_amount
            if token_out_amount < min_token_out_amount {
                return Err(StablePoolError::InsufficientOutputAmount);
            };
            // update reserves
            self.pool.reserves[token_in_id] =
                Self::rated_amount_to_amount(swap_res.new_source_amount, rate, token_in_id)?;
            self.pool.reserves[token_out_id] =
                Self::rated_amount_to_amount(swap_res.new_destination_amount, rate, token_out_id)?;
            // mint fees for admin
            // "Because amp_coef may change over time, we can't
            // determine the fees only when depositing/withdrawing
            // liquidity, admin fees must be minted and distributed on every swap"
            if fee_to.is_some() && swap_res.admin_fee > 0 {
                let admin_fee =
                    Self::rated_amount_to_amount(swap_res.admin_fee, rate, token_out_id)?;
                let mut admin_deposit_amounts = vec![0u128; self.pool.tokens.len()];
                admin_deposit_amounts[token_out_id] = admin_fee;
                // calc shares from admin fee
                let (admin_fee_lp, _) = math::compute_lp_amount_for_deposit(
                    &admin_deposit_amounts,
                    &self.pool.reserves,
                    self.psp22.total_supply(),
                    None,
                    self.amp_coef()?,
                )?;
                // mint fee (shares) to admin
                let events = self.psp22.mint(fee_to.unwrap(), admin_fee_lp)?;
                self.emit_events(events);
                // update reserve again
                self.pool.reserves[token_out_id] = self.pool.reserves[token_out_id]
                    .checked_add(admin_fee)
                    .ok_or(MathError::AddOverflow(1))?;
            }
            Ok((
                token_out_amount,
                Self::rated_amount_to_amount(
                    self.to_token_amount(token_out_id, swap_res.fee)?,
                    rate,
                    token_out_id,
                )?,
            ))
        }

        fn amount_to_rated_amount(amount: u128, rate: u128, id: usize) -> Result<u128, MathError> {
            if id == 0 {
                Ok(casted_mul(amount, rate)
                    .checked_div(ONE_AZERO.into())
                    .ok_or(MathError::DivByZero(1))?
                    .as_u128())
            } else {
                Ok(amount)
            }
        }

        fn rated_amount_to_amount(
            rated_amount: u128,
            rate: u128,
            id: usize,
        ) -> Result<u128, MathError> {
            if id == 0 {
                Ok(casted_mul(rated_amount, ONE_AZERO)
                    .checked_div(rate.into())
                    .ok_or(MathError::DivByZero(1))?
                    .as_u128())
            } else {
                Ok(rated_amount)
            }
        }

        fn rated_reserves(&self, rate: u128) -> Result<Vec<u128>, MathError> {
            let mut reserves = self.pool.reserves.clone();
            reserves[0] = Self::amount_to_rated_amount(reserves[0], rate, 0)?;
            Ok(reserves)
        }

        fn update_sazero_rate(&mut self) -> u128 {
            let rate = self
                .pool
                .sazero_rate_contract
                .get_azero_from_shares(ONE_SAZERO);
            self.pool.cached_sazero_rate = rate;
            self.pool.last_sazero_rate_update_ts = self.env().block_timestamp();
            rate
        }

        fn get_actual_sazero_rate(&self) -> u128 {
            if self
                .pool
                .last_sazero_rate_update_ts
                .checked_add(RATE_EXPIRE_TS)
                .unwrap()
                > self.env().block_timestamp()
            {
                self.pool.cached_sazero_rate
            } else {
                self.pool
                    .sazero_rate_contract
                    .get_azero_from_shares(ONE_SAZERO)
            }
        }

        fn get_actual_sazero_rate_and_update(&mut self) -> u128 {
            if self
                .pool
                .last_sazero_rate_update_ts
                .checked_add(RATE_EXPIRE_TS)
                .unwrap()
                < self.env().block_timestamp()
            {
                self.update_sazero_rate()
            } else {
                self.pool.cached_sazero_rate
            }
        }
    }

    impl StablePool for SazeroAzeroPairContract {
        #[ink(message)]
        fn add_liquidity(
            &mut self,
            min_share_amount: u128,
            amounts: Vec<u128>,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            if amounts.len() != self.pool.tokens.len() {
                return Err(StablePoolError::IncorrectAmountsCount);
            }
            // transfer amounts
            let mut c_amounts: Vec<u128> = Vec::new();
            for (id, &token) in self.pool.tokens.iter().enumerate() {
                self.token_by_address(token).transfer_from(
                    self.env().caller(),
                    self.env().account_id(),
                    amounts[id],
                    vec![],
                )?;
                c_amounts.push(self.to_comperable_amount(id, amounts[id])?);
            }
            let rate = self.get_actual_sazero_rate_and_update();
            let mut rated_c_amounts = c_amounts.clone();
            rated_c_amounts[0] = Self::amount_to_rated_amount(c_amounts[0], rate, 0)?;
            // calc lp tokens (shares_to_mint, fee)
            let (shares, fee_part) = math::compute_lp_amount_for_deposit(
                &rated_c_amounts,
                &self.rated_reserves(rate)?,
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef()?,
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
            for (i, &amount) in c_amounts.iter().enumerate() {
                self.pool.reserves[i] = self.pool.reserves[i]
                    .checked_add(amount)
                    .ok_or(MathError::AddOverflow(1))?;
            }
            Ok((shares, fee_part))
        }

        #[ink(message)]
        fn remove_liquidity(
            &mut self,
            max_share_amount: u128,
            amounts: Vec<u128>,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            if amounts.len() != self.pool.tokens.len() {
                return Err(StablePoolError::IncorrectAmountsCount);
            }
            // calc comparable amounts
            let rate = self.get_actual_sazero_rate_and_update();
            let c_amounts = self.to_comperable_amounts(&amounts)?;
            let mut rated_c_amounts = c_amounts.clone();
            rated_c_amounts[0] = Self::amount_to_rated_amount(c_amounts[0], rate, 0)?;
            let (shares_to_burn, fee_part) = math::compute_lp_amount_for_withdraw(
                &rated_c_amounts,
                &self.rated_reserves(rate)?,
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef()?,
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
            for (i, &amount) in c_amounts.iter().enumerate() {
                self.pool.reserves[i] = self.pool.reserves[i]
                    .checked_add(amount)
                    .ok_or(MathError::AddOverflow(1))?;
            }
            Ok((shares_to_burn, fee_part))
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
            //check token ids
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;
            // transfer token_in
            self.token_by_address(token_in).transfer_from(
                self.env().caller(),
                self.env().account_id(),
                token_in_amount,
                vec![],
            )?;
            // convert token_in_amount to comparable amount
            let comparable_token_in_amount =
                self.to_comperable_amount(token_in_id as usize, token_in_amount)?;
            // calculate amount out, mint admin fee and update reserves
            let (token_out_amount, swap_fee) = self._swap(
                token_in_id,
                token_out_id,
                comparable_token_in_amount,
                min_token_out_amount,
            )?;
            // transfer token_out
            self.token_by_address(token_out)
                .transfer(to, token_out_amount, vec![])?;
            Ok((token_out_amount, swap_fee))
        }

        #[ink(message)]
        fn swap_excess(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            min_token_out_amount: u128,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            //check token ids
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;
            // convert the excess of token_in to comperable amount
            let comparable_token_in_amount = self
                .to_comperable_amount(
                    token_in_id as usize,
                    self.token_by_address(token_in)
                        .balance_of(self.env().account_id()),
                )?
                .checked_sub(self.pool.reserves[token_in_id as usize])
                .ok_or(MathError::SubUnderflow(1))?;
            // calculate amount out, mint admin fee and update reserves
            let (token_out_amount, swap_fee) = self._swap(
                token_in_id,
                token_out_id,
                comparable_token_in_amount,
                min_token_out_amount,
            )?;
            // transfer token_out
            self.token_by_address(token_out)
                .transfer(to, token_out_amount, vec![])?;
            Ok((token_out_amount, swap_fee))
        }

        #[ink(message)]
        fn ramp_amp_coef(
            &mut self,
            target_amp_coef: u128,
            ramp_duration: u64,
        ) -> Result<(), StablePoolError> {
            self.ensure_onwer()?;
            self.pool.amp_coef.ramp_amp_coef(
                target_amp_coef,
                ramp_duration,
                self.env().block_timestamp(),
            )
        }

        #[ink(message)]
        fn set_owner(&mut self, new_owner: AccountId) -> Result<(), StablePoolError> {
            self.ensure_onwer()?;
            self.owner = new_owner;
            Ok(())
        }
    }

    impl StablePoolView for SazeroAzeroPairContract {
        #[ink(message)]
        fn tokens(&self) -> Vec<AccountId> {
            self.pool.tokens.clone()
        }

        #[ink(message)]
        fn reserves(&self) -> Vec<u128> {
            self.pool.reserves.clone()
        }
        #[ink(message)]
        fn amp_coef(&self) -> Result<u128, StablePoolError> {
            let current_time = self.env().block_timestamp();
            Ok(self.pool.amp_coef.compute_amp_coef(current_time)?)
        }

        #[ink(message)]
        fn get_swap_amount_out(
            &self,
            token_in: AccountId,
            token_out: AccountId,
            token_in_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;
            let rate = self.get_actual_sazero_rate();
            let rated_token_in_amount =
                Self::amount_to_rated_amount(token_in_amount, rate, token_in_id)?;
            let res = math::swap_to(
                token_in_id,
                self.to_comperable_amount(token_in_id, rated_token_in_amount)?,
                token_out_id,
                &self.rated_reserves(rate)?,
                &self.pool.fees,
                self.amp_coef()?,
                self.fee_to().is_some(),
            )?;
            Ok((
                Self::rated_amount_to_amount(
                    self.to_token_amount(token_out_id, res.amount_swapped)?,
                    rate,
                    token_out_id,
                )?,
                Self::rated_amount_to_amount(
                    self.to_token_amount(token_out_id, res.fee)?,
                    rate,
                    token_in_id,
                )?,
            ))
        }

        #[ink(message)]
        fn get_swap_amount_in(
            &self,
            token_in: AccountId,
            token_out: AccountId,
            token_out_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;
            let rate = self.get_actual_sazero_rate();
            let rated_token_out_amount =
                Self::amount_to_rated_amount(token_out_amount, rate, token_out_id)?;
            let res = math::swap_from(
                token_in_id,
                self.to_comperable_amount(token_out_id, rated_token_out_amount)?,
                token_out_id,
                &self.rated_reserves(rate)?,
                &self.pool.fees,
                self.amp_coef()?,
                self.fee_to().is_some(),
            )?;
            Ok((
                Self::rated_amount_to_amount(
                    self.to_token_amount(token_in_id, res.amount_swapped)?,
                    rate,
                    token_in_id,
                )?,
                Self::rated_amount_to_amount(
                    self.to_token_amount(token_out_id, res.fee)?,
                    rate,
                    token_in_id,
                )?,
            ))
        }

        #[ink(message)]
        fn get_mint_liquidity_for_amounts(
            &self,
            amounts: Vec<u128>,
        ) -> Result<(u128, u128), StablePoolError> {
            if amounts.len() != self.pool.tokens.len() {
                return Err(StablePoolError::IncorrectAmountsCount);
            }
            let mut rated_amounts = self.to_comperable_amounts(&amounts)?;
            let rate = self.get_actual_sazero_rate();
            rated_amounts[0] = Self::amount_to_rated_amount(rated_amounts[0], rate, 0)?;
            math::compute_lp_amount_for_deposit(
                &rated_amounts,
                &self.rated_reserves(rate)?,
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef()?,
            )
            .map_err(|err| StablePoolError::MathError(err))
        }

        #[ink(message)]
        fn get_amounts_for_liquidity_mint(
            &self,
            liquidity: u128,
        ) -> Result<Vec<u128>, StablePoolError> {
            let rate = self.get_actual_sazero_rate();
            match math::compute_deposit_amounts_for_lp(
                liquidity,
                &self.rated_reserves(rate)?,
                self.psp22.total_supply(),
            ) {
                Ok((mut amounts, _)) => {
                    amounts[0] = Self::rated_amount_to_amount(amounts[0], rate, 0)?;
                    Ok(self.to_token_amounts(&amounts)?)
                }
                Err(err) => Err(StablePoolError::MathError(err)),
            }
        }

        #[ink(message)]
        fn get_burn_liquidity_for_amounts(
            &self,
            amounts: Vec<u128>,
        ) -> Result<(u128, u128), StablePoolError> {
            if amounts.len() != self.pool.tokens.len() {
                return Err(StablePoolError::IncorrectAmountsCount);
            }
            let mut rated_amounts = self.to_comperable_amounts(&amounts)?;
            let rate = self.get_actual_sazero_rate();
            rated_amounts[0] = Self::amount_to_rated_amount(rated_amounts[0], rate, 0)?;
            math::compute_lp_amount_for_withdraw(
                &rated_amounts,
                &self.rated_reserves(rate)?,
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef()?,
            )
            .map_err(|err| StablePoolError::MathError(err))
        }

        #[ink(message)]
        fn get_amounts_for_liquidity_burn(
            &self,
            liquidity: u128,
        ) -> Result<Vec<u128>, StablePoolError> {
            let rate = self.get_actual_sazero_rate();
            match math::compute_withdraw_amounts_for_lp(
                liquidity,
                &self.rated_reserves(rate)?,
                self.psp22.total_supply(),
            ) {
                Ok((mut amounts, _)) => {
                    amounts[0] = Self::rated_amount_to_amount(amounts[0], rate, 0)?;
                    Ok(self.to_token_amounts(&amounts)?)
                }
                Err(err) => Err(StablePoolError::MathError(err)),
            }
        }
    }

    impl PSP22 for SazeroAzeroPairContract {
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

    impl PSP22Metadata for SazeroAzeroPairContract {
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
            self.decimals
        }
    }

    // #[cfg(test)]
    // mod test {
    //     use ink::primitives::AccountId;

    //     use super::*;
    //     #[test]
    //     fn amount_to_comperable_and_back_1() {
    //         let stable_pool_contract = SazeroAzeroPairContract::new(
    //             vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])],
    //             vec![6, 12],
    //             1,
    //             AccountId::from([0u8; 32]),
    //             AccountId::from([0u8; 32]),
    //         );
    //         let amount: u128 = 1_000_000_000_000; // 1000000.000000
    //         let expect_amount: u128 = amount * 10u128.pow(6); // 1000000.000000000000000000
    //         assert_eq!(
    //             stable_pool_contract.to_comperable_amount(0, amount),
    //             Ok(expect_amount)
    //         );
    //         assert_eq!(
    //             stable_pool_contract.to_token_amount(0, expect_amount),
    //             Ok(amount)
    //         );
    //         let amount: u128 = 1_000_000_000_000_000_000; // 1000000.000000000000
    //         let expect_amount: u128 = amount;
    //         assert_eq!(
    //             stable_pool_contract.to_comperable_amount(1, amount),
    //             Ok(expect_amount)
    //         );
    //         assert_eq!(
    //             stable_pool_contract.to_token_amount(1, expect_amount),
    //             Ok(amount)
    //         );
    //     }

    //     #[test]
    //     fn amount_to_comperable_and_back_2() {
    //         let stable_pool_contract = SazeroAzeroPairContract::new(
    //             vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])],
    //             vec![0, 24],
    //             1,
    //             AccountId::from([0u8; 32]),
    //             AccountId::from([0u8; 32]),
    //         );
    //         let amount: u128 = 1_000_000; // 1000000
    //         let expect_amount: u128 = amount * 10u128.pow(24); // 1000000.000000000000000000000000
    //         assert_eq!(
    //             stable_pool_contract.to_comperable_amount(0, amount),
    //             Ok(expect_amount)
    //         );
    //         assert_eq!(
    //             stable_pool_contract.to_token_amount(0, expect_amount),
    //             Ok(amount)
    //         );
    //         let amount: u128 = 1_000_000_000_000_000_000_000_000_000_000; // 1000000.000000000000000000000000
    //         let expect_amount: u128 = amount;
    //         assert_eq!(
    //             stable_pool_contract.to_comperable_amount(1, amount),
    //             Ok(expect_amount)
    //         );
    //         assert_eq!(
    //             stable_pool_contract.to_token_amount(1, expect_amount),
    //             Ok(amount)
    //         );
    //     }

    //     #[test]
    //     fn amount_to_comperable_and_back_3() {
    //         let stable_pool_contract = SazeroAzeroPairContract::new(
    //             vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])],
    //             vec![1, 18],
    //             1,
    //             AccountId::from([0u8; 32]),
    //             AccountId::from([0u8; 32]),
    //         );
    //         let amount: u128 = 1_000_000_0; // 1000000.0
    //         let expect_amount: u128 = amount * 10u128.pow(17); // 1000000.000000000000000000
    //         assert_eq!(
    //             stable_pool_contract.to_comperable_amount(0, amount),
    //             Ok(expect_amount)
    //         );
    //         assert_eq!(
    //             stable_pool_contract.to_token_amount(0, expect_amount),
    //             Ok(amount)
    //         );
    //         let amount: u128 = 1_000_000_000_000_000_000_000_000; // 1000000.00000000000000000
    //         let expect_amount: u128 = amount; // 1000000.000000000000000000
    //         assert_eq!(
    //             stable_pool_contract.to_comperable_amount(1, amount),
    //             Ok(expect_amount)
    //         );
    //         assert_eq!(
    //             stable_pool_contract.to_token_amount(1, expect_amount),
    //             Ok(amount)
    //         );
    //     }
    // }
}
