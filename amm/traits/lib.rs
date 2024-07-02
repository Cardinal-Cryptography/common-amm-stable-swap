#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod rate_provider;
mod stable_pool;

pub type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;

pub use amm_helpers::math::MathError;
pub use rate_provider::RateProvider;
pub use stable_pool::{StablePool, StablePoolError, StablePoolView};
