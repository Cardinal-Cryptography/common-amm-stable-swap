/// Address for which the private key is unknown.
/// This is used for sending MINIMUM_LIQUIDITY when minting tokens in Pair contract.
/// Result of sha512 hashing the ZERO_ADDERSS_MSG to curve (curve25519).
pub const BURN_ADDRESS: [u8; 32] = [
    58, 108, 115, 140, 64, 55, 232, 71, 183, 215, 14, 149, 138, 148, 201, 178, 212, 197, 99, 60,
    250, 175, 203, 88, 227, 37, 36, 127, 63, 212, 16, 72,
];

#[allow(unused)]
const BURN_ADDRESS_MSG: &str = "This is Aleph Zero DEX's zero address.";

/// Minimum liquidity threshold that is subtracted
/// from the minted liquidity and sent to the `BURN_ADDRESS`.
/// Prevents price manipulation and saturation.
/// See UniswapV2 whitepaper for more details.
/// NOTE: This value is taken from UniswapV2 whitepaper and is correct
/// only for liquidity tokens with precision = 18.
pub const MINIMUM_LIQUIDITY: u128 = 1000;

pub mod stable_pool {
    // amount * 0.06%
    pub const TRADE_FEE_BPS: u32 = 6;
    // amount * 0.06% * 20% (part of the TRADE_FEE)
    pub const ADMIN_FEE_BPS: u32 = 2_000;

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

#[cfg(test)]
mod burn_address {
    use curve25519_dalek::ristretto::RistrettoPoint;
    use sha2::Sha512;

    use super::BURN_ADDRESS_MSG;

    #[test]
    fn test_burn_address() {
        let p = RistrettoPoint::hash_from_bytes::<Sha512>(BURN_ADDRESS_MSG.as_bytes());
        let burn_address = p.compress();
        assert_eq!(super::BURN_ADDRESS, burn_address.to_bytes());
    }
}
