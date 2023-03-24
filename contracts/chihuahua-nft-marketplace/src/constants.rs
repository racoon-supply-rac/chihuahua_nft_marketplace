use cosmwasm_std::{Uint128};

pub const MARKETPLACE_USDC_INDICATOR: &str = "TOTAL_USDC";

pub const MIN_PRICE: Uint128 = Uint128::new(10_000u128);
pub const MAX_PRICE: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

// Max expiration set to 120 days
pub const MAX_EXPIRATION_SECONDS: u64 = 31_536_000u64;
pub const MIN_EXPIRATION_SECONDS: u64 = 86_400u64;

pub const MAX_NFT_PER_COLLECTION: u32 = 10_000u32;