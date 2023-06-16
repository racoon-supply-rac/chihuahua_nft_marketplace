use std::marker::PhantomData;

use cosmwasm_std::{
    to_binary, Addr, Deps, Empty, Order, QueryRequest, StdResult, Uint128, WasmQuery,
};
use cw721::{Cw721QueryMsg, TokensResponse};
use cw721_base::helpers::Cw721Contract;
use cw_storage_plus::Bound;

use nft_marketplace_utils::config::{Config, ConfigRewardGenStatsMsg};
use nft_marketplace_utils::marketplace_statistics::MarketplaceStatsByDenom;
use nft_marketplace_utils::nft_collection::{
    nft_collection_denoms, NftCollectionAddress, NftCollectionAddressTokenId,
    NftCollectionInfoAndUsdcVol, NftCollectionInfoByDenom, NftContractInfo, NftContractType,
    TokenId,
};
use nft_marketplace_utils::nft_offer::{nft_offers, NftOffer};
use nft_marketplace_utils::nft_sale::{
    define_unique_collection_nft_id, nfts_for_sale, NftSale, TokenSaleHistory, TokensAndIfSaleInfo,
};
use nft_marketplace_utils::profile::Profile;

use crate::constants::{MARKETPLACE_USDC_INDICATOR, MAX_NFT_PER_COLLECTION};
use crate::state::{
    CONFIG, GENERAL_STATS, LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL, MARKETPLACE_STATS_BY_DENOM,
    PROFILES, REWARD_SYSTEM, TOKEN_SALE_HISTORY, USERNAMES,
};

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
        general_stats,
    })
}

pub fn query_nft_collection(
    deps: Deps,
    nft_collection_address: NftCollectionAddress,
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
    let output_by_denom = config
        .accepted_ibc_denominations
        .list_of_denoms
        .iter()
        .map(|accepted_denom| MARKETPLACE_STATS_BY_DENOM.load(deps.storage, accepted_denom))
        .collect::<StdResult<Vec<MarketplaceStatsByDenom>>>()
        .unwrap_or_default();
    Ok(output_by_denom)
}

pub fn query_marketplace_total_volume(deps: Deps) -> StdResult<Uint128> {
    let volume: Uint128 = LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL
        .load(deps.storage, MARKETPLACE_USDC_INDICATOR)?
        .usdc_volume;
    Ok(volume)
}

pub fn query_nft_collection_total_volume(
    deps: Deps,
    nft_collection_address: NftCollectionAddress,
) -> StdResult<Uint128> {
    let maybe_volume: Option<NftCollectionInfoAndUsdcVol> =
        LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL.may_load(deps.storage, &nft_collection_address)?;
    Ok(maybe_volume
        .unwrap_or(NftCollectionInfoAndUsdcVol {
            nft_collection_address: "".to_string(),
            nft_contract_info: NftContractInfo {
                code_id: 0,
                nft_contract_type: NftContractType::Cw2981MultiRoyalties,
            },
            usdc_volume: Uint128::zero(),
        })
        .usdc_volume)
}

pub fn query_nft_collection_type(
    deps: Deps,
    nft_collection_address: NftCollectionAddress,
) -> StdResult<NftContractInfo> {
    let nft_contract_type: NftContractInfo = LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL
        .load(deps.storage, &nft_collection_address)?
        .nft_contract_info;
    Ok(nft_contract_type)
}

pub fn query_nft_for_sale(
    deps: Deps,
    collection_address: NftCollectionAddress,
    token_id: String,
) -> StdResult<NftSale> {
    let collection_token_id_unique: String =
        define_unique_collection_nft_id(&collection_address, &token_id);
    let nft_for_sale = nfts_for_sale().may_load(deps.storage, collection_token_id_unique)?;
    if let Some(nft_for_sale) = nft_for_sale {
        Ok(nft_for_sale)
    } else {
        Ok(NftSale {
            seller: "".to_string(),
            nft_collection_address: "".to_string(),
            token_id: "".to_string(),
            sale_price_value: Default::default(),
            sale_price_denom: "".to_string(),
            sale_expiration: Default::default(),
        })
    }
}

pub fn query_nft_trade_history(
    deps: Deps,
    nft_collection_address: NftCollectionAddress,
    token_id: TokenId,
) -> StdResult<Vec<TokenSaleHistory>> {
    let collection_token_id_unique: String =
        define_unique_collection_nft_id(&nft_collection_address, &token_id);
    let nft_trade_history = TOKEN_SALE_HISTORY.load(deps.storage, &collection_token_id_unique);
    if let Ok(result) = nft_trade_history {
        Ok(result)
    } else {
        Ok(vec![])
    }
}

pub fn query_profile_info(deps: Deps, address_or_username: String) -> StdResult<Profile> {
    let address: String;
    let username: String;
    let profile: Option<Profile>;
    if deps.api.addr_validate(&address_or_username).is_err() {
        // if it is an error -> will be username
        username = address_or_username;
        let address_result = USERNAMES.load(deps.storage, &username)?;
        address = (*address_result).parse().unwrap();
        profile = PROFILES.may_load(deps.storage, &address)?;
    } else {
        address = address_or_username;
        profile = PROFILES.may_load(deps.storage, &address)?;
    }
    Ok(profile.unwrap_or(Profile {
        address,
        username: None,
        vip_level: None,
        profile_nft_collection: None,
        profile_nft_token_id: None,
        background_nft_collection: None,
        background_nft_token_id: None,
        description: None,
        nft_showcase: None,
        links: None,
        profile_messages: None,
        number_of_trades: None,
        buy_info: None,
        sell_info: None,
        display_trade_info: None,
    }))
}

pub fn query_nfts_for_sale_from_seller(
    deps: Deps,
    seller: String,
    start_after_collection_token_id: Option<NftCollectionAddressTokenId>,
    output_length: Option<u32>,
) -> StdResult<Vec<NftSale>> {
    let max_size = output_length
        .unwrap_or(DEFAULT_OUTPUT_LENGTH)
        .min(MAX_OUTPUT_LENGTH) as usize;
    let nfts_for_sale_info = nfts_for_sale()
        .idx
        .seller_index
        .prefix(seller)
        .range(
            deps.storage,
            Some(Bound::exclusive(
                start_after_collection_token_id.unwrap_or_default(),
            )),
            None,
            Order::Ascending,
        )
        .take(max_size)
        .map(|std_result| std_result.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(nfts_for_sale_info)
}

pub fn query_all_tokens_by_coll_and_if_sale(
    deps: Deps,
    address: String,
    nft_collection_address: NftCollectionAddress,
    output_length: Option<u32>,
) -> StdResult<Vec<TokensAndIfSaleInfo>> {
    let max_size = output_length
        .unwrap_or(DEFAULT_OUTPUT_LENGTH)
        .min(MAX_OUTPUT_LENGTH) as usize;

    let nfts_for_sale_info = nfts_for_sale()
        .idx
        .collection_seller_index
        .prefix((nft_collection_address.clone(), address.clone()))
        .range(deps.storage, None, None, Order::Ascending)
        .take(max_size)
        .map(|std_result| std_result.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>();

    if nfts_for_sale_info.is_err() {
        return Ok(vec![TokensAndIfSaleInfo {
            token_id: "".to_string(),
            for_sale: false,
        }]);
    }
    let nfts_for_sale_info = nfts_for_sale_info.unwrap();

    let all_tokens: StdResult<TokensResponse> =
        deps.querier
            .query::<TokensResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: nft_collection_address,
                msg: to_binary(&Cw721QueryMsg::Tokens {
                    owner: address,
                    start_after: None,
                    limit: Some(max_size as u32),
                })?,
            }));

    if all_tokens.is_err() {
        return Ok(vec![TokensAndIfSaleInfo {
            token_id: "".to_string(),
            for_sale: false,
        }]);
    }
    let all_tokens = all_tokens.unwrap();

    let mut output_matched: Vec<TokensAndIfSaleInfo> = Vec::with_capacity(all_tokens.tokens.len());
    for token in all_tokens.tokens.iter() {
        let is_for_sale = nfts_for_sale_info
            .iter()
            .any(|nft_sale| &nft_sale.token_id == token);
        let token_and_if_sale: TokensAndIfSaleInfo = TokensAndIfSaleInfo {
            token_id: token.clone(),
            for_sale: is_for_sale,
        };
        output_matched.push(token_and_if_sale);
    }

    Ok(output_matched)
}

pub fn query_nft_offers_from_offerer(
    deps: Deps,
    offerer: String,
    start_after: Option<(NftCollectionAddress, TokenId)>,
    output_length: Option<u32>,
) -> StdResult<Vec<NftOffer>> {
    let max_size = output_length
        .unwrap_or(DEFAULT_OUTPUT_LENGTH)
        .min(MAX_OUTPUT_LENGTH) as usize;
    let mut start_after_valid: Option<Bound<String>> = None;
    if let Some((nft_collection_address, token_id)) = start_after {
        let unique_id = define_unique_collection_nft_id(&nft_collection_address, &token_id);
        start_after_valid = Some(Bound::exclusive(unique_id));
    }
    let nfts_for_sale_info = nft_offers()
        .idx
        .offerer_index
        .prefix(offerer)
        .range(deps.storage, start_after_valid, None, Order::Ascending)
        .take(max_size)
        .map(|std_result| std_result.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(nfts_for_sale_info)
}

pub fn query_token_ids_by_collection(
    deps: Deps,
    address: String,
    nft_collection_addresses: Vec<NftCollectionAddress>,
) -> StdResult<Vec<(NftCollectionAddress, Vec<String>)>> {
    let mut output = Vec::with_capacity(nft_collection_addresses.len());
    for nft_coll in nft_collection_addresses.iter() {
        let tokens_response = Cw721Contract::<Empty, Empty>(
            deps.api.addr_validate(nft_coll)?,
            PhantomData,
            PhantomData,
        )
        .tokens(
            &deps.querier,
            address.clone(),
            None,
            Some(MAX_NFT_PER_COLLECTION),
        )?;
        output.push((nft_coll.clone(), tokens_response.tokens))
    }
    Ok(output)
}

pub fn query_nft_offers_by_token_id(
    deps: Deps,
    nft_collection_address: NftCollectionAddress,
    token_id: TokenId,
    start_after: Option<(NftCollectionAddress, TokenId)>,
    output_length: Option<u32>,
) -> StdResult<Vec<NftOffer>> {
    let max_size = output_length
        .unwrap_or(DEFAULT_OUTPUT_LENGTH)
        .min(MAX_OUTPUT_LENGTH) as usize;
    let prefix = define_unique_collection_nft_id(&nft_collection_address, &token_id);
    let mut start_after_valid: Option<Bound<String>> = None;
    if let Some((nft_collection_address, token_id)) = start_after {
        let unique_id = define_unique_collection_nft_id(&nft_collection_address, &token_id);
        start_after_valid = Some(Bound::exclusive(unique_id));
    }
    let nfts_for_sale_info = nft_offers()
        .idx
        .collection_tokenid_index
        .prefix(prefix)
        .range(deps.storage, start_after_valid, None, Order::Ascending)
        .take(max_size)
        .map(|std_result| std_result.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(nfts_for_sale_info)
}

pub fn query_nfts_for_sale_from_collection(
    deps: Deps,
    nft_collection_address: NftCollectionAddress,
    start_after_token_id: Option<TokenId>,
    output_length: Option<u32>,
) -> StdResult<Vec<NftSale>> {
    let max_size = output_length
        .unwrap_or(DEFAULT_OUTPUT_LENGTH)
        .min(MAX_OUTPUT_LENGTH) as usize;
    let collection_token_id_unique: String = define_unique_collection_nft_id(
        &nft_collection_address,
        &start_after_token_id.unwrap_or_default(),
    );
    let nfts_for_sale_info = nfts_for_sale()
        .idx
        .collection_index
        .prefix(nft_collection_address)
        .range(
            deps.storage,
            Some(Bound::exclusive(collection_token_id_unique)),
            None,
            Order::Ascending,
        )
        .take(max_size)
        .map(|std_result| std_result.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(nfts_for_sale_info)
}
