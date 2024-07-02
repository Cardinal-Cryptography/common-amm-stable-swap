pub mod stable_pool {
    // Token amounts are rescaled so as if they have TOKEN_TARGET_DECIMALS decimal places.
    pub const TOKEN_TARGET_DECIMALS: u8 = 18;
    pub const TOKEN_TARGET_PRECISION: u128 = 10u128.pow(TOKEN_TARGET_DECIMALS as u32);

    // Precision for rate values. If the rate is 1.2, the rate provider should return 1.2 * RATE_PRECISION.
    pub const RATE_DECIMALS: u8 = 12;
    pub const RATE_PRECISION: u128 = 10u128.pow(RATE_DECIMALS as u32);

    /// Min amplification coefficient.
    pub const MIN_AMP: u128 = 1;
    /// Max amplification coefficient.
    pub const MAX_AMP: u128 = 1_000_000;
}
