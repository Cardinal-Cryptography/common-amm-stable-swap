#[ink::trait_definition]
pub trait RateProvider {
    // Get "rate" of a paritucular token with respect to a given base token.
    // For instance, in the context of liquid staking, the base token could be the native token of the chain and the rate,
    // at a particular point of time would be the price of the yield bearing liquid staking token in terms of the base token.
    // The rate is supposed to have precision of TARGET_DECIMALS=18 decimal places. So if the rate is 1.5, it should be represented as 1.5 * 10^24.
    #[ink(message)]
    fn get_rate(&mut self) -> u128;
}
