#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod sazero_rate_mock {

    const ONE_SHARE: u128 = 10u128.pow(18);
    const ONE_AZERO: u128 = 10u128.pow(12);

    #[ink(storage)]
    pub struct SazeroMockContract {
        initial_ts: u64,
    }

    impl SazeroMockContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                initial_ts: Self::env().block_timestamp(),
            }
        }

        /// Calculate the value of sAZERO in terms of AZERO
        #[ink(message)]
        pub fn get_azero_from_shares(&self, shares: u128) -> u128 {
            // mock increasing rate (0.01% every 1 minute)
            let now = self.env().block_timestamp();
            let time_d = (now - self.initial_ts) as u128; // ms elapsed
            let current_one_share_price = ONE_AZERO + ONE_AZERO * time_d / 600000000;
            current_one_share_price * shares / ONE_SHARE
        }
    }

    #[cfg(test)]
    mod test {
        use ink::env::{test::set_block_timestamp, DefaultEnvironment};

        use super::*;
        #[test]
        fn test_1() {
            let minute: u64 = 60 * 1000;
            let rate_contract = SazeroMockContract::new();
            set_block_timestamp::<DefaultEnvironment>(minute * 10000); // after 10000 minutes should be doubled
            assert_eq!(
                rate_contract.get_azero_from_shares(ONE_SHARE),
                ONE_AZERO * 2
            );
        }
    }
}
