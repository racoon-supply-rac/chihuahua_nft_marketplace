use std::str::FromStr;

use cosmwasm_std::{to_binary, Decimal, Deps, QuerierWrapper, QueryRequest, Uint128, WasmQuery};
use cw721::{Cw721QueryMsg, TokensResponse};

use cw2981_multiroyalties::msg::Cw2981QueryMsg;
use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{InvalidNftCollection, InvalidRoyalty};
use nft_marketplace_utils::legacy_nft_metadata::Cw2981LegacyMetadata;
use nft_marketplace_utils::nft_collection::{NftCollectionInfoAndUsdcVol, NftContractType};
use nft_marketplace_utils::nft_sale::NftSale;
use nft_marketplace_utils::response_handler::RoyaltiesInfoResponse;

use crate::constants::MADHUAHUA_NFTS;

pub fn validate_contract_type_and_royalty(
    nft_collection_info: NftCollectionInfoAndUsdcVol,
    querier: QuerierWrapper,
) -> Result<(), ContractError> {
    match nft_collection_info.nft_contract_info.nft_contract_type {
        // So far only Cw2981MultiRoyalties has a royalty check (as the Madhuahua was custom)
        NftContractType::Cw2981MultiRoyalties => {
            let nft_tokens: TokensResponse =
                querier.query::<TokensResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: nft_collection_info.nft_collection_address.clone(),
                    msg: to_binary(&Cw721QueryMsg::AllTokens {
                        start_after: None,
                        limit: Some(1),
                    })?,
                }))?;

            let _nft_royalties: Vec<RoyaltiesInfoResponse> = querier
                .query::<Vec<RoyaltiesInfoResponse>>(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: nft_collection_info.nft_collection_address,
                    msg: to_binary(&cw2981_multiroyalties::QueryMsg::Extension {
                        msg: Cw2981QueryMsg::RoyaltyInfo {
                            token_id: nft_tokens.tokens[0].clone(),
                            sale_price: Uint128::new(100_000u128),
                        },
                    })?,
                }))?;
        }
        NftContractType::Cw2981MadHuahua => {
            if nft_collection_info.nft_collection_address != MADHUAHUA_NFTS {
                return Err(ContractError::NftMarketplaceError(InvalidNftCollection {}));
            }
        }
        NftContractType::Cw721OnChainMetadata => {
            // TODO
        }
        NftContractType::MarketplaceInfo => {
            return Err(ContractError::NftMarketplaceError(InvalidNftCollection {}));
        }
    }
    Ok(())
}

pub fn compute_royalty(
    nft_for_sale_info: NftSale,
    nft_collection_info: NftCollectionInfoAndUsdcVol,
    deps: Deps,
) -> Result<Vec<RoyaltiesInfoResponse>, ContractError> {
    let mut nft_royalties: Vec<RoyaltiesInfoResponse> = vec![];
    match nft_collection_info.nft_contract_info.nft_contract_type {
        // So far only Cw2981MultiRoyalties has a royalty check (as the Madhuahua was custom)
        NftContractType::Cw2981MultiRoyalties => {
            nft_royalties =
                deps.querier
                    .query::<Vec<RoyaltiesInfoResponse>>(&QueryRequest::Wasm(WasmQuery::Smart {
                        contract_addr: nft_for_sale_info.nft_collection_address.clone(),
                        msg: to_binary(&cw2981_multiroyalties::QueryMsg::Extension {
                            msg: Cw2981QueryMsg::RoyaltyInfo {
                                token_id: nft_for_sale_info.token_id.clone(),
                                sale_price: nft_for_sale_info.sale_price_value,
                            },
                        })?,
                    }))?;
        }
        NftContractType::Cw2981MadHuahua => {
            let nft_metadata_for_royalties: cw721::NftInfoResponse<Cw2981LegacyMetadata> = deps
                .querier
                .query::<cw721::NftInfoResponse<Cw2981LegacyMetadata>>(&QueryRequest::Wasm(
                    WasmQuery::Smart {
                        contract_addr: nft_for_sale_info.nft_collection_address.clone(),
                        msg: to_binary(&cw721::Cw721QueryMsg::NftInfo {
                            token_id: nft_for_sale_info.token_id,
                        })?,
                    },
                ))?;
            for royalty in nft_metadata_for_royalties
                .extension
                .royalty_info
                .unwrap()
                .into_iter()
            {
                if Decimal::permille(royalty.thousands.clone().parse::<u64>().unwrap())
                    > Decimal::from_str("1.0").unwrap()
                    || Decimal::permille(royalty.thousands.clone().parse::<u64>().unwrap())
                        < Decimal::from_str("0.001").unwrap()
                {
                    return Err(ContractError::NftMarketplaceError(InvalidRoyalty {}));
                }
                let royalty_amount: Uint128 = nft_for_sale_info.sale_price_value
                    * Decimal::permille(royalty.thousands.clone().parse::<u64>().unwrap());
                nft_royalties.push(RoyaltiesInfoResponse {
                    address: deps.api.addr_validate(&royalty.address.to_string())?,
                    royalty_amount,
                })
            }
        }
        _ => {}
    }
    Ok(nft_royalties)
}
