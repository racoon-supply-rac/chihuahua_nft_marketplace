use cw_storage_plus::{Item, Map};
use price_oracle_utils::config::Config;
use price_oracle_utils::oracle::OraclePrices;

pub const CONFIG: Item<Config> = Item::new("config");
pub const HISTORICAL_PRICES: Map<u64, OraclePrices> = Map::new("historical_prices");