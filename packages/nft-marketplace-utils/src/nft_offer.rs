use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint128};
use general_utils::denominations::{Denomination};
use crate::nft_collection::{NftCollectionAddress, TokenId};

#[cw_serde]
pub struct NftOffer {
    pub offerer_address: String,
    pub nft_collection_address: NftCollectionAddress,
    pub token_id: TokenId,
    pub offer_price_value: Uint128,
    pub offer_price_denom: Denomination,
    pub offer_expiration: Timestamp
}