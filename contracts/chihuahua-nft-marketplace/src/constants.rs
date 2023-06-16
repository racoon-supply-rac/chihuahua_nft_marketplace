use cosmwasm_std::Uint128;

pub const MARKETPLACE_USDC_INDICATOR: &str = "TOTAL_USDC";

pub const MIN_PRICE: Uint128 = Uint128::new(100u128);
pub const MAX_PRICE: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

// Max expiration set to 120 days
pub const MAX_EXPIRATION_SECONDS: u64 = 31_536_000u64;
pub const MIN_EXPIRATION_SECONDS: u64 = 86_400u64;

pub const MAX_NFT_PER_COLLECTION: u32 = 10_000u32;

pub const MADHUAHUA_NFTS: &str =
    "chihuahua1xv4zzcs3hqfwjfcpaq3swtj9unfa7qa4km00rxclwg98zuvw82tsnwxnw7";
pub const SANCTUARY_NFTS: &str =
    "chihuahua1s6uhncxycfakk27ja765rmt05g5zzxjtw0kx3pad64077w0aar7qm64akt";
