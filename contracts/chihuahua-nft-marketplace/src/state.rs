use cw_storage_plus::{Item, Map};

use nft_marketplace_utils::config::Config;
use nft_marketplace_utils::marketplace_statistics::{GeneralStats, MarketplaceStatsByDenom};
use nft_marketplace_utils::nft_collection::NftCollectionInfoAndUsdcVol;
use nft_marketplace_utils::nft_sale::TokenSaleHistory;
use nft_marketplace_utils::profile::Profile;
use nft_marketplace_utils::reward_system::RewardSystem;

pub const CONFIG: Item<Config> = Item::new("config");

pub const REWARD_SYSTEM: Item<RewardSystem> = Item::new("reward_system");

pub const MARKETPLACE_STATS_BY_DENOM: Map<&str, MarketplaceStatsByDenom> =
    Map::new("marketplace_stats_by_denom");

pub const LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL: Map<&str, NftCollectionInfoAndUsdcVol> =
    Map::new("listed_nft_collections_info_and_usdc_vol");

pub const TOKEN_SALE_HISTORY: Map<&str, Vec<TokenSaleHistory>> = Map::new("token_sale_history");

pub const PROFILES: Map<&str, Profile> = Map::new("profiles");

pub const USERNAMES: Map<&str, String> = Map::new("usernames");

pub const GENERAL_STATS: Item<GeneralStats> = Item::new("general_stats");
