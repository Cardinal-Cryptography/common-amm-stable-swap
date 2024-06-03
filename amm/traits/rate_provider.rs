#[ink::trait_definition]
pub trait RateProvider {
    // Get "rate" of a paritucular token with respect to a given base token.
    // For instance, in the context of liquid staking, the base token could be the native token of the chain and the rate,
    // at a particular point of time would be the price of the yield bearing liquid staking token in terms of the base token.
    #[ink(message)]
    fn get_rate(&mut self) -> u128;
}
