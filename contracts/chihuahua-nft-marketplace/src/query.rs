use std::marker::PhantomData;
use cosmwasm_std::{Addr, Deps, Empty, Order, StdError, StdResult, Uint128};
use cw721_base::helpers::Cw721Contract;
use cw_storage_plus::Bound;
use nft_marketplace_utils::config::{Config, ConfigRewardGenStatsMsg};
use nft_marketplace_utils::marketplace_statistics::MarketplaceStatsByDenom;
use nft_marketplace_utils::nft_collection::{nft_collection_denoms, NftCollectionInfoByDenom,
                                            NftCollectionAddress, NftCollectionAddressTokenId, TokenId};
use nft_marketplace_utils::nft_sale::{define_unique_collection_nft_id, NftSale, nfts_for_sale, TokenSaleHistory};
use nft_marketplace_utils::nft_offer::{NftOffer};
use nft_marketplace_utils::profile::Profile;
use crate::constants::{MARKETPLACE_USDC_INDICATOR, MAX_NFT_PER_COLLECTION};
use crate::state::{CONFIG, GENERAL_STATS, MARKETPLACE_STATS_BY_DENOM,
                   NFT_COLLECTION_VOLUME_USDC, PROFILES, REWARD_SYSTEM, TOKEN_SALE_HISTORY};

const MAX_OUTPUT_LENGTH: u32 = 500;
const DEFAULT_OUTPUT_LENGTH: u32 = 20;

pub fn query_config(deps: Deps) -> StdResult<ConfigRewardGenStatsMsg> {
    let config = CONFIG.load(deps.storage)?;
    let reward_system = REWARD_SYSTEM.load(deps.storage)?;
    let general_stats = GENERAL_STATS.load(deps.storage)?;
    Ok(ConfigRewardGenStatsMsg {
        contract_enabled: config.contract_enabled,
        contract_owner: config.contract_owner,
        accepted_ibc_denominations: config.accepted_ibc_denominations,
        marketplace_pct_fees: config.marketplace_pct_fees,
        marketplace_listing_fee_value: config.marketplace_listing_fee_value,
        marketplace_listing_fee_denom: config.marketplace_listing_fee_denom,
        oracle_contract_address: config.oracle_contract_address,
        reward_system,
        general_stats
    })
}

pub fn query_nft_collection(
    deps: Deps,
    nft_collection_address: NftCollectionAddress
) -> StdResult<Vec<NftCollectionInfoByDenom>> {
    let nft_collection_denoms_info = nft_collection_denoms()
        .idx
        .collection_index
        .prefix(Addr::unchecked(nft_collection_address))
        .range(deps.storage, None, None, Order::Ascending)
        .map(|std_result| std_result.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(nft_collection_denoms_info)
}

pub fn query_marketplace_info(deps: Deps) -> StdResult<Vec<MarketplaceStatsByDenom>> {
    let config: Config = CONFIG.load(deps.storage)?;
    let output_by_denom = config.accepted_ibc_denominations
        .list_of_denoms
        .iter()
        .map(|accepted_denom| MARKETPLACE_STATS_BY_DENOM.load(deps.storage, accepted_denom))
        .collect::<StdResult<Vec<MarketplaceStatsByDenom>>>()
        .unwrap_or_default();
    Ok(output_by_denom)
}

pub fn query_marketplace_total_volume(deps: Deps) -> StdResult<Uint128> {
    let volume: Uint128 = NFT_COLLECTION_VOLUME_USDC.load(deps.storage, MARKETPLACE_USDC_INDICATOR)?;
    Ok(volume)
}

pub fn query_nft_collection_total_volume(deps: Deps, nft_collection_address: NftCollectionAddress) -> StdResult<Uint128> {
    let volume: Uint128 = NFT_COLLECTION_VOLUME_USDC.load(deps.storage, &nft_collection_address.to_string())?;
    Ok(volume)
}

pub fn query_nft_for_sale(
    deps: Deps,
    collection_address: NftCollectionAddress,
    token_id: String) -> StdResult<NftSale> {
    let collection_token_id_unique: String = define_unique_collection_nft_id(
        &collection_address,
        &token_id,
    );
    let nft_for_sale = nfts_for_sale().may_load(deps.storage, collection_token_id_unique)?;
    if nft_for_sale.is_none() {
        Err(StdError::GenericErr { msg: "NftNotForSale".to_string() })
    } else {
        Ok(nft_for_sale.unwrap())
    }
}

pub fn query_nft_trade_history(
    deps: Deps,
    nft_collection_address: NftCollectionAddress,
    token_id: TokenId
) -> StdResult<Vec<TokenSaleHistory>> {
    let collection_token_id_unique: String = define_unique_collection_nft_id(
        &nft_collection_address,
        &token_id,
    );
    let nft_trade_history = TOKEN_SALE_HISTORY.load(deps.storage, &collection_token_id_unique)?;
    Ok(nft_trade_history)
}

pub fn query_profile_info(deps: Deps, address: String) -> StdResult<Profile> {
    let profile_info = PROFILES.load(deps.storage, &address.to_string())?;
    Ok(profile_info)
}

pub fn query_nfts_for_sale_from_seller(
    deps: Deps,
    seller: String,
    start_after_collection_token_id: Option<NftCollectionAddressTokenId>,
    output_length: Option<u32>
) -> StdResult<Vec<NftSale>> {
    let max_size = output_length.unwrap_or(DEFAULT_OUTPUT_LENGTH).min(MAX_OUTPUT_LENGTH) as usize;
    let nfts_for_sale_info = nfts_for_sale()
        .idx
        .seller_index
        .prefix(seller)
        .range(deps.storage, Some(
            Bound::exclusive(start_after_collection_token_id.unwrap_or_default())), None, Order::Ascending)
        .take(max_size)
        .map(|std_result| std_result.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(nfts_for_sale_info)
}

pub fn query_nft_offers_from_offerer(
    _deps: Deps,
    _offerer: String,
    _start_after: Option<(NftCollectionAddress, TokenId)>,
    _output_length: Option<u32>
) -> StdResult<Vec<NftOffer>> {
    Ok(vec![])
}

pub fn query_token_ids_by_collection(
    deps: Deps,
    address: String,
    nft_collection_addresses: Vec<NftCollectionAddress>,
) -> StdResult<Vec<(NftCollectionAddress, Vec<String>)>> {
    let mut output = Vec::with_capacity(nft_collection_addresses.len());
    for nft_coll in nft_collection_addresses.iter() {
        let tokens_response = Cw721Contract::<Empty, Empty>(
            deps.api.addr_validate(&nft_coll)?,
            PhantomData,
            PhantomData)
            .tokens(
                &deps.querier,
                address.clone(),
                None,
                Some(MAX_NFT_PER_COLLECTION)
            )?;
        output.push((nft_coll.clone(), tokens_response.tokens))
    }
    Ok(output)
}

pub fn query_nft_offers_by_token_id(
    _deps: Deps,
    _nft_collection_address: NftCollectionAddress,
    _token_id: TokenId,
    _start_after: Option<(NftCollectionAddress, TokenId)>,
    _output_length: Option<u32>,
) -> StdResult<Vec<NftOffer>> {
    Ok(vec![])
}

pub fn query_nfts_for_sale_from_collection(
    deps: Deps,
    nft_collection_address: NftCollectionAddress,
    start_after_token_id: Option<TokenId>,
    output_length: Option<u32>
) -> StdResult<Vec<NftSale>> {
    let max_size = output_length.unwrap_or(DEFAULT_OUTPUT_LENGTH).min(MAX_OUTPUT_LENGTH) as usize;
    let collection_token_id_unique: String = define_unique_collection_nft_id(
        &nft_collection_address,
        &start_after_token_id.unwrap_or_default()
    );
    let nfts_for_sale_info = nfts_for_sale()
        .idx
        .collection_index
        .prefix(nft_collection_address.to_string())
        .range(deps.storage, Some(
            Bound::exclusive(collection_token_id_unique)), None, Order::Ascending)
        .take(max_size)
        .map(|std_result| std_result.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(nfts_for_sale_info)
}