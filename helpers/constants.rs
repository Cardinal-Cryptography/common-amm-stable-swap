pub mod stable_pool {
    // Token amounts are rescaled so as if they have TOKEN_TARGET_DECIMALS decimal places.
    pub const TOKEN_TARGET_DECIMALS: u8 = 18;
    pub const TOKEN_TARGET_PRECISION: u128 = 10u128.pow(TOKEN_TARGET_DECIMALS as u32);

    // Precision for rate values. If the rate is 1.2, the rate provider should return 1.2 * RATE_PRECISION.
    pub const RATE_DECIMALS: u8 = 12;
    pub const RATE_PRECISION: u128 = 10u128.pow(RATE_DECIMALS as u32);

    /// Given as an integer with 1e9 precision (1%)
    pub const MAX_TRADE_FEE: u32 = 10_000_000;
    /// Given as an integer with 1e9 precision (50%)
    ///
    /// It is taken as part of the trade fee thus,
    /// a maximum 50% of 1% goes to the protocol (0.5% of the trade)
    pub const MAX_PROTOCOL_FEE: u32 = 500_000_000;
    /// Fee denominator
    pub const FEE_DENOM: u32 = 1_000_000_000;

    /// Maximum number coins (PSP22 token contracts) in the pool.
    pub const MAX_COINS: usize = 8;

    /// Minimum ramp duration, in milisec (24h).
    pub const MIN_RAMP_DURATION: u64 = 86400000;
    /// Min amplification coefficient.
    pub const MIN_AMP: u128 = 1;
    /// Max amplification coefficient.
    pub const MAX_AMP: u128 = 1_000_000;
    /// Max amplification change (how many times it can increase/decrease compared to current value).
    pub const MAX_AMP_CHANGE: u128 = 10;
}
