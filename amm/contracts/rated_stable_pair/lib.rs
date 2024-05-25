#![cfg_attr(not(feature = "std"), no_std, no_main)]
mod token_rate;

#[ink::contract]
pub mod rated_stable_pair {
    // 0.0006 * amount
    pub const TRADE_FEE_BPS: u32 = 6;
    // 0.0006 * 0.2 * amount (part of the TRADE_FEE)
    pub const ADMIN_FEE_BPS: u32 = 2_000;

    use amm_helpers::{
        ensure,
        stable_swap_math::{self as math, amp_coef::*, fees::Fees},
    };
    use ink::contract_ref;
    use ink::prelude::{
        string::{String, ToString},
        {vec, vec::Vec},
    };
    use psp22::{PSP22Data, PSP22Error, PSP22Event, PSP22Metadata, PSP22};
    use traits::{Factory, MathError, StablePool, StablePoolError, StablePoolView};

    use crate::token_rate::*;

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
    pub struct RatedStablePairData {
        /// Factory contract
        factory: contract_ref!(Factory),
        /// List of tokens. Token at id 0 is the rated token.
        tokens: Vec<AccountId>,
        /// Tokens precision factors used for normalization.
        precisions: Vec<u128>,
        /// Reserves in comparable amounts.
        c_reserves: Vec<u128>,
        /// Amplification coefficient.
        amp_coef: AmplificationCoefficient,
        /// Fees
        fees: Fees,
        /// Rate of the token at id 0
        token_0_rate: TokenRate,
    }

    #[ink(storage)]
    pub struct RatedStablePairContract {
        owner: AccountId,
        pool: RatedStablePairData,
        psp22: PSP22Data,
        decimals: u8,
    }

    impl RatedStablePairContract {
        #[ink(constructor)]
        pub fn new(
            token_0_rated: AccountId,
            token_1: AccountId,
            token_0_decimals: u8,
            token_1_decimals: u8,
            token_0_rate_contract: AccountId,
            init_amp_coef: u128,
            factory: AccountId,
            owner: AccountId,
        ) -> Result<Self, StablePoolError> {
            ensure!(token_0_rated != token_1, StablePoolError::IdenticalTokenId);
            let c_reserves = vec![0, 0];
            let max_decimals = token_0_decimals.max(token_1_decimals);
            let precisions = vec![
                10u128.pow(max_decimals.checked_sub(token_0_decimals).unwrap().into()),
                10u128.pow(max_decimals.checked_sub(token_1_decimals).unwrap().into()),
            ];
            Ok(Self {
                owner,
                pool: RatedStablePairData {
                    factory: factory.into(),
                    tokens: vec![token_0_rated, token_1],
                    precisions,
                    c_reserves,
                    amp_coef: AmplificationCoefficient::new(init_amp_coef)?,
                    fees: Fees::new(TRADE_FEE_BPS, ADMIN_FEE_BPS),
                    token_0_rate: TokenRate::new(
                        Self::env().block_timestamp(),
                        token_0_rate_contract,
                    ),
                },
                psp22: PSP22Data::default(),
                decimals: SAZERO_DECIMALS,
            })
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
        fn to_comparable_amount(&self, amount: u128, token_id: usize) -> Result<u128, MathError> {
            amount
                .checked_mul(self.pool.precisions[token_id])
                .ok_or(MathError::MulOverflow(101))
        }

        /// Converts provided comparable `amount` to token amount
        fn to_token_amount(&self, amount: u128, token_id: usize) -> u128 {
            // it is safe to unwrap since precision for any token is >= 1
            amount.checked_div(self.pool.precisions[token_id]).unwrap()
        }

        /// Converts provided tokens `amounts` to comparable amounts
        fn to_token_amounts(&self, amounts: &[u128]) -> Vec<u128> {
            let mut token_amounts: Vec<u128> = Vec::new();
            for (id, &amount) in amounts.iter().enumerate() {
                token_amounts.push(self.to_token_amount(amount, id));
            }
            token_amounts
        }

        /// Converts provided comparable `amounts` to tokens amounts
        fn to_comparable_amounts(&self, amounts: &[u128]) -> Result<Vec<u128>, MathError> {
            let mut comparable_amounts: Vec<u128> = Vec::new();
            for (id, &amount) in amounts.iter().enumerate() {
                comparable_amounts.push(self.to_comparable_amount(amount, id)?);
            }
            Ok(comparable_amounts)
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

        fn get_current_token_0_rate(&self) -> TokenRate {
            let mut rate = self.pool.token_0_rate.clone();
            rate.update(self.env().block_timestamp());
            rate
        }

        fn update_token_0_rate(&mut self) {
            self.pool.token_0_rate.update(self.env().block_timestamp());
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
            c_token_in_amount: u128,
            min_token_out_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            if c_token_in_amount == 0 {
                return Err(StablePoolError::InsufficientInputAmount);
            }
            // rate amount in
            self.update_token_0_rate();
            let rate = &self.pool.token_0_rate;
            let r_c_token_in_amount =
                rate.amount_to_rated_amount(c_token_in_amount, token_in_id)?;
            // calc amount_out and fees
            let (r_c_token_out_amount, r_c_fee) = math::swap_to(
                token_in_id,
                r_c_token_in_amount,
                token_out_id,
                &rate.amounts_to_rated_amounts(&self.pool.c_reserves)?,
                &self.pool.fees,
                self.amp_coef()?,
                // fee_to.is_some(),
            )?;
            let c_token_out_amount =
                rate.rated_amount_to_amount(r_c_token_out_amount, token_out_id)?;
            let token_out_amount = self.to_token_amount(c_token_out_amount, token_out_id);
            // Check if swapped amount is not less than min_token_out_amount
            if token_out_amount < min_token_out_amount {
                return Err(StablePoolError::InsufficientOutputAmount);
            };
            // update reserves
            self.pool.c_reserves[token_in_id] = self.pool.c_reserves[token_in_id]
                .checked_add(c_token_in_amount)
                .ok_or(MathError::AddOverflow(101))?;
            self.pool.c_reserves[token_out_id] = self.pool.c_reserves[token_out_id]
                .checked_sub(c_token_out_amount)
                .ok_or(MathError::SubUnderflow(101))?;

            // distribute admin fee
            if let Some(fee_to) = self.fee_to() {
                let r_c_admin_fee = self.pool.fees.admin_trade_fee(r_c_fee)?;
                if r_c_admin_fee > 0 {
                    let mut r_c_admin_deposit_amounts = vec![0u128; self.pool.tokens.len()];
                    r_c_admin_deposit_amounts[token_out_id] = r_c_admin_fee;
                    let mut r_c_reserves = rate.amounts_to_rated_amounts(&self.pool.c_reserves)?;
                    r_c_reserves[token_out_id] = r_c_reserves[token_out_id]
                        .checked_sub(r_c_admin_fee)
                        .ok_or(MathError::SubUnderflow(102))?;
                    let (admin_fee_lp, _) = math::compute_lp_amount_for_deposit(
                        &r_c_admin_deposit_amounts,
                        &r_c_reserves,
                        self.psp22.total_supply(),
                        None, // no fees
                        self.amp_coef()?,
                    )?;
                    // mint fee (shares) to admin
                    let events = self.psp22.mint(fee_to, admin_fee_lp)?;
                    self.emit_events(events);
                }
            }
            Ok((
                token_out_amount,
                rate.rated_amount_to_amount(
                    self.to_token_amount(r_c_fee, token_out_id),
                    token_out_id,
                )?,
            ))
        }
    }

    impl StablePool for RatedStablePairContract {
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
                c_amounts.push(self.to_comparable_amount(amounts[id], id)?);
            }
            // get updated rate
            self.update_token_0_rate();
            let rate = &self.pool.token_0_rate;
            let rated_c_amounts = rate.amounts_to_rated_amounts(&c_amounts)?;
            // calc lp tokens (shares_to_mint, fee)
            let (shares, fee_part) = math::compute_lp_amount_for_deposit(
                &rated_c_amounts,
                &rate.amounts_to_rated_amounts(&self.pool.c_reserves)?,
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
                self.pool.c_reserves[i] = self.pool.c_reserves[i]
                    .checked_add(amount)
                    .ok_or(MathError::AddOverflow(102))?;
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
            if amounts.len() != self.pool.tokens.len() {
                return Err(StablePoolError::IncorrectAmountsCount);
            }
            // calc comparable amounts
            let c_amounts = self.to_comparable_amounts(&amounts)?;
            // get updated rate
            self.update_token_0_rate();
            let rate = &self.pool.token_0_rate;
            let rated_c_amounts = rate.amounts_to_rated_amounts(&c_amounts)?;
            let (shares_to_burn, fee_part) = math::compute_lp_amount_for_withdraw(
                &rated_c_amounts,
                &rate.amounts_to_rated_amounts(&self.pool.c_reserves)?,
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
                self.pool.c_reserves[i] = self.pool.c_reserves[i]
                    .checked_add(amount)
                    .ok_or(MathError::AddOverflow(103))?;
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
            let c_token_in_amount = self.to_comparable_amount(token_in_amount, token_in_id)?;
            // calculate amount out, mint admin fee and update reserves
            let (token_out_amount, swap_fee) = self._swap(
                token_in_id,
                token_out_id,
                c_token_in_amount,
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
        fn swap_excess(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            min_token_out_amount: u128,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            //check token ids
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;
            // convert the excess of token_in to comparable amount
            let c_token_in_amount = self
                .to_comparable_amount(
                    self.token_by_address(token_in)
                        .balance_of(self.env().account_id()),
                    token_in_id,
                )?
                .checked_sub(self.pool.c_reserves[token_in_id])
                .ok_or(MathError::SubUnderflow(103))?;
            // calculate amount out, mint admin fee and update reserves
            let (token_out_amount, swap_fee) = self._swap(
                token_in_id,
                token_out_id,
                c_token_in_amount,
                min_token_out_amount,
            )?;
            // transfer token_out
            self.token_by_address(token_out)
                .transfer(to, token_out_amount, vec![])?;
            self.env().emit_event(Swap {
                sender: self.env().caller(),
                token_in,
                amount_in: self.to_token_amount(c_token_in_amount, token_in_id),
                token_out,
                amount_out: token_out_amount,
                to,
            });
            Ok((token_out_amount, swap_fee))
        }

        #[ink(message)]
        fn ramp_amp_coef(
            &mut self,
            target_amp_coef: u128,
            ramp_duration: u64,
        ) -> Result<(), StablePoolError> {
            self.ensure_onwer()?;
            let current_amp_coef = self.amp_coef()?;
            self.pool
                .amp_coef
                .ramp_amp_coef(target_amp_coef, ramp_duration, self.env().block_timestamp())
                .map_err(|err| StablePoolError::AmpCoefError(err))?;
            self.env().emit_event(RampAmpCoef {
                old_amp_coef: current_amp_coef,
                new_amp_coef: target_amp_coef,
                init_time: self.env().block_timestamp(),
                ramp_duration,
            });
            Ok(())
        }

        #[ink(message)]
        fn set_owner(&mut self, new_owner: AccountId) -> Result<(), StablePoolError> {
            self.ensure_onwer()?;
            self.owner = new_owner;
            Ok(())
        }
    }

    impl StablePoolView for RatedStablePairContract {
        #[ink(message)]
        fn tokens(&self) -> Vec<AccountId> {
            self.pool.tokens.clone()
        }

        #[ink(message)]
        fn reserves(&self) -> Vec<u128> {
            self.pool.c_reserves.clone()
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
            let rate = self.get_current_token_0_rate();
            let r_c_token_in_amount = rate.amount_to_rated_amount(
                self.to_comparable_amount(token_in_amount, token_in_id)?,
                token_in_id,
            )?;
            let (r_c_token_out_amount, r_c_fee) = math::swap_to(
                token_in_id,
                r_c_token_in_amount,
                token_out_id,
                &rate.amounts_to_rated_amounts(&self.pool.c_reserves)?,
                &self.pool.fees,
                self.amp_coef()?,
            )?;
            Ok((
                rate.rated_amount_to_amount(
                    self.to_token_amount(r_c_token_out_amount, token_out_id),
                    token_out_id,
                )?,
                rate.rated_amount_to_amount(
                    self.to_token_amount(r_c_fee, token_out_id),
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
            let rate = self.get_current_token_0_rate();
            let r_c_token_out_amount = rate.amount_to_rated_amount(
                self.to_comparable_amount(token_out_amount, token_out_id)?,
                token_out_id,
            )?;
            let (r_c_token_in_amount, r_c_fee) = math::swap_from(
                token_in_id,
                r_c_token_out_amount,
                token_out_id,
                &rate.amounts_to_rated_amounts(&self.pool.c_reserves)?,
                &self.pool.fees,
                self.amp_coef()?,
            )?;
            Ok((
                rate.rated_amount_to_amount(
                    self.to_token_amount(r_c_token_in_amount, token_in_id),
                    token_in_id,
                )?,
                rate.rated_amount_to_amount(
                    self.to_token_amount(r_c_fee, token_out_id),
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
            let rate = self.get_current_token_0_rate();
            let rated_amounts =
                rate.amounts_to_rated_amounts(&self.to_comparable_amounts(&amounts)?)?;
            math::compute_lp_amount_for_deposit(
                &rated_amounts,
                &rate.amounts_to_rated_amounts(&self.pool.c_reserves)?,
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
            let rate = self.get_current_token_0_rate();
            match math::compute_deposit_amounts_for_lp(
                liquidity,
                &rate.amounts_to_rated_amounts(&self.pool.c_reserves)?,
                self.psp22.total_supply(),
            ) {
                Ok((amounts, _)) => {
                    Ok(self.to_token_amounts(&rate.rated_amounts_to_amounts(&amounts)?))
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
            let rate = self.get_current_token_0_rate();
            let rated_amounts =
                rate.amounts_to_rated_amounts(&self.to_comparable_amounts(&amounts)?)?;
            math::compute_lp_amount_for_withdraw(
                &rated_amounts,
                &rate.amounts_to_rated_amounts(&self.pool.c_reserves)?,
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
            let rate = self.get_current_token_0_rate();
            match math::compute_withdraw_amounts_for_lp(
                liquidity,
                &rate.amounts_to_rated_amounts(&self.pool.c_reserves)?,
                self.psp22.total_supply(),
            ) {
                Ok((amounts, _)) => {
                    Ok(self.to_token_amounts(&rate.rated_amounts_to_amounts(&amounts)?))
                }
                Err(err) => Err(StablePoolError::MathError(err)),
            }
        }
    }

    impl PSP22 for RatedStablePairContract {
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

    impl PSP22Metadata for RatedStablePairContract {
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
}
