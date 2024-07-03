#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod sazero_rate_mock {
    use amm_helpers::constants::stable_pool::RATE_PRECISION;
    #[ink(storage)]
    pub struct SazeroMockContract {
        initial_ts: u64,
    }

    impl SazeroMockContract {
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {
                initial_ts: Self::env().block_timestamp(),
            }
        }
    }

    impl traits::RateProvider for SazeroMockContract {
        /// Calculate the value of sAZERO in terms of AZERO with TARGET_DECIMALS precision
        #[ink(message)]
        fn get_rate(&mut self) -> u128 {
            // mock increasing rate (0.01% every 1 minute)
            let now = self.env().block_timestamp();
            let time_d = (now - self.initial_ts) as u128; // ms elapsed
            RATE_PRECISION + RATE_PRECISION * time_d / 600000000
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use ink::env::{test::set_block_timestamp, DefaultEnvironment};
        use traits::RateProvider;
        #[test]
        fn test_1() {
            let minute: u64 = 60 * 1000;
            let mut rate_contract = SazeroMockContract::new();
            set_block_timestamp::<DefaultEnvironment>(minute * 10000); // after 10000 minutes should be doubled
            assert_eq!(rate_contract.get_rate(), super::RATE_PRECISION * 2);
        }
    }
}
