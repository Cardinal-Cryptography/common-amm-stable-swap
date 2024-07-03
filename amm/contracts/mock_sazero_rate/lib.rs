#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod mock_sazero_rate {
    #[ink(storage)]
    pub struct MockSazeroRateContract {
        rate: u128,
    }

    impl MockSazeroRateContract {
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self { rate: 10u128.pow(12u32) }
        }

        #[ink(message)]
        pub fn set_rate(&mut self, rate: u128) {
            self.rate = rate;
        }
    }

    impl traits::RateProvider for MockSazeroRateContract {
        #[ink(message)]
        fn get_rate(&mut self) -> u128 {
            self.rate
        }
    }
}
