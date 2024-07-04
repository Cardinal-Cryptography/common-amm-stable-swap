use ink::{contract_ref, env::DefaultEnvironment, primitives::AccountId};
use scale::{Decode, Encode};
use traits::RateProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ExternalTokenRate {
    cached_token_rate: u128,
    last_token_rate_update_ts: u64,
    token_rate_contract: AccountId,
    expiration_duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum TokenRate {
    Constant(u128),
    External(ExternalTokenRate),
}

impl TokenRate {
    pub fn new_constant(rate: u128) -> Self {
        Self::Constant(rate)
    }

    pub fn new_external(
        current_time: u64,
        token_rate_contract: AccountId,
        expiration_duration_ms: u64,
    ) -> Self {
        let mut rate = Self::External(ExternalTokenRate::new(
            current_time,
            token_rate_contract,
            expiration_duration_ms,
        ));
        _ = rate.update_rate_no_cache(current_time);
        rate
    }

    // To make sure the rate is up-to-date, the caller should call `update_rate` before calling this method.
    pub fn get_rate(&self) -> u128 {
        match self {
            Self::Constant(rate) => *rate,
            Self::External(external) => external.get_rate(),
        }
    }

    // [nit] Some docs would be helpful here. This method returns if the update has been done, right?
    // But what if we're Self::External, we are outdated, but the returned rate is the same as it was previously?
    // Currently, we'll return true, which might be confusing.
    pub fn update_rate(&mut self, current_time: u64) -> bool {
        match self {
            Self::External(external) => external.update_rate(current_time),
            // [nit] Exhaustive match would be more future-proof.
            Self::Constant(_) => false,
        }
    }

    pub fn update_rate_no_cache(&mut self, current_time: u64) -> bool {
        match self {
            Self::External(external) => external.update_rate_no_cache(current_time),
            // [nit] Exhaustive match would be more future-proof.
            Self::Constant(_) => false,
        }
    }
}

impl ExternalTokenRate {
    pub fn new(
        current_time: u64,
        token_rate_contract: AccountId,
        expiration_duration_ms: u64,
    ) -> Self {
        let rate = Self::query_rate(token_rate_contract);
        Self {
            cached_token_rate: rate,
            last_token_rate_update_ts: current_time,
            token_rate_contract,
            expiration_duration_ms,
        }
    }

    pub fn get_rate(&self) -> u128 {
        self.cached_token_rate
    }

    pub fn update_rate(&mut self, current_time: u64) -> bool {
        if self.is_outdated(current_time) {
            self.update(current_time)
        } else {
            false
        }
    }

    pub fn update_rate_no_cache(&mut self, current_time: u64) -> bool {
        self.update(current_time)
    }

    fn query_rate(token_rate_contract: AccountId) -> u128 {
        let mut rate_provider: contract_ref!(RateProvider, DefaultEnvironment) =
            token_rate_contract.into();
        rate_provider.get_rate()
    }

    fn is_outdated(&self, current_time: u64) -> bool {
        current_time.saturating_sub(self.last_token_rate_update_ts) >= self.expiration_duration_ms
    }

    fn update(&mut self, current_time: u64) -> bool {
        self.cached_token_rate = Self::query_rate(self.token_rate_contract);
        self.last_token_rate_update_ts = current_time;
        true
    }
}
