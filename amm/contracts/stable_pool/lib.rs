#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod stable_pool {
    // amount * 0.06%
    pub const TRADE_FEE_BPS: u32 = 6;
    // amount * 0.06% * 20% (part of the TRADE_FEE)
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
        /// Factory contract
        factory: contract_ref!(Factory),
        /// Tokens.
        tokens: Vec<AccountId>,
        /// Tokens precision that token amounts are multiplied by in order to adjust to comparable amounts (common precision).
        precisions: Vec<u128>,
        /// Reserves in comparable amounts.
        c_reserves: Vec<u128>,
        /// Amplification coefficient.
        amp_coef: AmplificationCoefficient,
        /// Fees
        fees: Fees,
    }

    #[ink(storage)]
    pub struct StablePoolContract {
        owner: AccountId,
        pool: StablePoolData,
        psp22: PSP22Data,
        decimals: u8,
    }

    impl StablePoolContract {
        #[ink(constructor)]
        pub fn new(
            tokens: Vec<AccountId>,
            tokens_decimals: Vec<u8>,
            init_amp_coef: u128,
            factory: AccountId,
            owner: AccountId,
        ) -> Result<Self, StablePoolError> {
            let mut unique_tokens = tokens.clone();
            unique_tokens.sort();
            unique_tokens.dedup();
            let token_count = tokens.len();
            ensure!(
                unique_tokens.len() == token_count,
                StablePoolError::IdenticalTokenId
            );
            ensure!(
                token_count == tokens_decimals.len(),
                StablePoolError::IncorrectTokenCount
            );
            let max_decimal = tokens_decimals.iter().max().unwrap_or(&0);
            let precisions = tokens_decimals
                .iter()
                .map(|&decimal| 10u128.pow(max_decimal.checked_sub(decimal).unwrap().into()))
                .collect();
            Ok(Self {
                owner,
                pool: StablePoolData {
                    factory: factory.into(),
                    tokens,
                    c_reserves: vec![0; token_count],
                    precisions,
                    amp_coef: AmplificationCoefficient::new(init_amp_coef)?,
                    fees: Fees::new(TRADE_FEE_BPS, ADMIN_FEE_BPS),
                },
                psp22: PSP22Data::default(),
                decimals: *max_decimal,
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

        #[inline]
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
            amounts
                .iter()
                .enumerate()
                .map(|(id, &amount)| self.to_token_amount(amount, id))
                .collect()
        }

        /// Converts provided comparable `amounts` to tokens amounts
        fn to_comparable_amounts(&self, amounts: &[u128]) -> Result<Vec<u128>, MathError> {
            amounts
                .iter()
                .enumerate()
                .map(|(id, &amount)| self.to_comparable_amount(amount, id))
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

        /// This method is for internal use only
        /// - calculates token_out amount
        /// - calculates swap fee
        /// - mints admin fee
        /// - updates reserves
        ///
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
            // calc amount_out and fees
            let (c_token_out_amount, c_fee) = math::swap_to(
                token_in_id,
                c_token_in_amount,
                token_out_id,
                &self.pool.c_reserves,
                &self.pool.fees,
                self.amp_coef()?,
            )?;
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
                // calc shares from admin fee
                let c_admin_fee = self.pool.fees.admin_trade_fee(c_fee)?;
                if c_admin_fee > 0 {
                    let mut c_admin_deposit_amounts = vec![0u128; self.pool.tokens.len()];
                    c_admin_deposit_amounts[token_out_id] = c_admin_fee;
                    let mut c_reserves = self.pool.c_reserves.clone();
                    c_reserves[token_out_id] = c_reserves[token_out_id]
                        .checked_sub(c_admin_fee)
                        .ok_or(MathError::SubUnderflow(102))?;
                    let (admin_fee_lp, _) = math::compute_lp_amount_for_deposit(
                        &c_admin_deposit_amounts,
                        &c_reserves,
                        self.psp22.total_supply(),
                        None, // no fees
                        self.amp_coef()?,
                    )?;
                    // mint fee (shares) to admin
                    let events = self.psp22.mint(fee_to, admin_fee_lp)?;
                    self.emit_events(events);
                }
            }
            Ok((token_out_amount, self.to_token_amount(c_fee, token_out_id)))
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
            // calc lp tokens (shares_to_mint, fee)
            let (shares, fee_part) = math::compute_lp_amount_for_deposit(
                &c_amounts,
                &self.pool.c_reserves,
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
            if amounts.len() != self.pool.tokens.len() {
                return Err(StablePoolError::IncorrectAmountsCount);
            }
            // calc comparable amounts
            let c_amounts = self.to_comparable_amounts(&amounts)?;
            let (shares_to_burn, fee_part) = math::compute_lp_amount_for_withdraw(
                &c_amounts,
                &self.pool.c_reserves,
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
                    .ok_or(MathError::AddOverflow(101))?;
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
            // convert the balance of token_in to comparable amount and sub c_reserve
            let c_token_in_amount = self
                .to_comparable_amount(
                    self.token_by_address(token_in)
                        .balance_of(self.env().account_id()),
                    token_in_id,
                )?
                .checked_sub(self.pool.c_reserves[token_in_id as usize])
                .ok_or(MathError::SubUnderflow(102))?;
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

    impl StablePoolView for StablePoolContract {
        #[ink(message)]
        fn tokens(&self) -> Vec<AccountId> {
            self.pool.tokens.clone()
        }

        #[ink(message)]
        fn reserves(&self) -> Vec<u128> {
            self.to_token_amounts(&self.pool.c_reserves)
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
            let (c_token_out_amount, c_fee) = math::swap_to(
                token_in_id as usize,
                self.to_comparable_amount(token_in_amount, token_in_id)?,
                token_out_id as usize,
                &self.pool.c_reserves,
                &self.pool.fees,
                self.amp_coef()?,
            )?;
            Ok((
                self.to_token_amount(c_token_out_amount, token_out_id),
                self.to_token_amount(c_fee, token_out_id),
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
            let (c_token_in_amount, c_fee) = math::swap_from(
                token_in_id as usize,
                self.to_comparable_amount(token_out_amount, token_out_id)?,
                token_out_id as usize,
                &self.pool.c_reserves,
                &self.pool.fees,
                self.amp_coef()?,
            )?;
            Ok((
                self.to_token_amount(c_token_in_amount, token_in_id),
                self.to_token_amount(c_fee, token_out_id),
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
            math::compute_lp_amount_for_deposit(
                &self.to_comparable_amounts(&amounts)?,
                &self.pool.c_reserves,
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
            match math::compute_deposit_amounts_for_lp(
                liquidity,
                &self.pool.c_reserves,
                self.psp22.total_supply(),
            ) {
                Ok((amounts, _)) => Ok(self.to_token_amounts(&amounts)),
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
            math::compute_lp_amount_for_withdraw(
                &self.to_comparable_amounts(&amounts)?,
                &self.pool.c_reserves,
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
            match math::compute_withdraw_amounts_for_lp(
                liquidity,
                &self.pool.c_reserves,
                self.psp22.total_supply(),
            ) {
                Ok((amounts, _)) => Ok(self.to_token_amounts(&amounts)),
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

    #[cfg(test)]
    mod test {
        use ink::primitives::AccountId;

        use super::*;
        #[test]
        fn amount_to_comparable_and_back_1() {
            let stable_pool_contract = StablePoolContract::new(
                vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])],
                vec![6, 12],
                1,
                AccountId::from([0u8; 32]),
                AccountId::from([0u8; 32]),
            )
            .map_err(|err| panic!("Contract instantiation error: {err:?}"))
            .unwrap();
            let amount: u128 = 1_000_000_000_000; // 1000000.000000
            let expect_amount: u128 = amount * 10u128.pow(6); // 1000000.000000000000000000
            assert_eq!(
                stable_pool_contract.to_comparable_amount(amount, 0),
                Ok(expect_amount)
            );
            assert_eq!(
                stable_pool_contract.to_token_amount(expect_amount, 0),
                amount
            );
            let amount: u128 = 1_000_000_000_000_000_000; // 1000000.000000000000
            let expect_amount: u128 = amount;
            assert_eq!(
                stable_pool_contract.to_comparable_amount(amount, 1),
                Ok(expect_amount)
            );
            assert_eq!(
                stable_pool_contract.to_token_amount(expect_amount, 1),
                amount
            );
        }

        #[test]
        fn amount_to_comparable_and_back_2() {
            let stable_pool_contract = StablePoolContract::new(
                vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])],
                vec![0, 24],
                1,
                AccountId::from([0u8; 32]),
                AccountId::from([0u8; 32]),
            )
            .map_err(|err| panic!("Contract instantiation error: {err:?}"))
            .unwrap();
            let amount: u128 = 1_000_000; // 1000000
            let expect_amount: u128 = amount * 10u128.pow(24); // 1000000.000000000000000000000000
            assert_eq!(
                stable_pool_contract.to_comparable_amount(amount, 0),
                Ok(expect_amount)
            );
            assert_eq!(
                stable_pool_contract.to_token_amount(expect_amount, 0),
                amount
            );
            let amount: u128 = 1_000_000_000_000_000_000_000_000_000_000; // 1000000.000000000000000000000000
            let expect_amount: u128 = amount;
            assert_eq!(
                stable_pool_contract.to_comparable_amount(amount, 1),
                Ok(expect_amount)
            );
            assert_eq!(
                stable_pool_contract.to_token_amount(expect_amount, 1),
                amount
            );
        }

        #[test]
        fn amount_to_comparable_and_back_3() {
            let stable_pool_contract = StablePoolContract::new(
                vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])],
                vec![1, 18],
                1,
                AccountId::from([0u8; 32]),
                AccountId::from([0u8; 32]),
            )
            .map_err(|err| panic!("Contract instantiation error: {err:?}"))
            .unwrap();
            let amount: u128 = 1_000_000_0; // 1000000.0
            let expect_amount: u128 = amount * 10u128.pow(17); // 1000000.000000000000000000
            assert_eq!(
                stable_pool_contract.to_comparable_amount(amount, 0),
                Ok(expect_amount)
            );
            assert_eq!(
                stable_pool_contract.to_token_amount(expect_amount, 0),
                amount
            );
            let amount: u128 = 1_000_000_000_000_000_000_000_000; // 1000000.00000000000000000
            let expect_amount: u128 = amount; // 1000000.000000000000000000
            assert_eq!(
                stable_pool_contract.to_comparable_amount(amount, 1),
                Ok(expect_amount)
            );
            assert_eq!(
                stable_pool_contract.to_token_amount(expect_amount, 1),
                amount
            );
        }
    }
}
