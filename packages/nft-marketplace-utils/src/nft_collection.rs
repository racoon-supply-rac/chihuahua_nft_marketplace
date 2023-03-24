use std::marker::PhantomData;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Empty, QuerierWrapper, QueryRequest, to_binary, Uint128, WasmQuery};
use cw721::{ContractInfoResponse, Cw721QueryMsg, NumTokensResponse};
use cw721_base::helpers::Cw721Contract;
use cw_storage_plus::{Index, IndexedMap, IndexList, MultiIndex};
use general_utils::denominations::Denomination;
use general_utils::error::ContractError;
use general_utils::error::NftCollectionError::NoNftsMintedForThisContract;
use crate::nft_sale::NftSale;

pub type NftCollectionAddress = String;
pub type TokenId = String;
pub type NftCollectionAddressTokenId = String;

#[cw_serde]
pub struct NftCollectionInfoByDenom {
    pub nft_collection_address: NftCollectionAddress,
    pub denom: Denomination,
    pub collection_name: String,
    pub nfts_for_sale: u64,
    pub realized_trades: u64,
    pub total_volume: Uint128,
    pub current_floor: Uint128
}

impl NftCollectionInfoByDenom {
    pub fn new_checked(
        deps_querier: QuerierWrapper,
        nft_collection_address: NftCollectionAddress,
        denom: Denomination,
    ) -> Result<Self, ContractError> {
        // Fetch and validate NFT contract information
        // TODO: Potentially a better way of doing it
        let given_contract_info: ContractInfoResponse = Cw721Contract::<Empty, Empty>(
            Addr::unchecked(nft_collection_address.clone().to_string()),
            PhantomData,
            PhantomData)
            .contract_info(&deps_querier)?;

        let number_of_tokens: NumTokensResponse =
            deps_querier
                .query::<NumTokensResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: nft_collection_address.clone().to_string(),
                    msg: to_binary(&Cw721QueryMsg::NumTokens {})?,
                }))?;

        if number_of_tokens.count < 1 {
            return Err(ContractError::NftCollection(NoNftsMintedForThisContract {}))
        }

        Ok(NftCollectionInfoByDenom {
            nft_collection_address,
            denom,
            collection_name: given_contract_info.name,
            nfts_for_sale: 0,
            realized_trades: 0,
            total_volume: Uint128::zero(),
            current_floor: Uint128::zero(),
        })
    }

    pub fn register_sale(self, nft_for_sale_validated: NftSale) -> Self {
        let current_floor = if self.current_floor == Uint128::zero() {
            nft_for_sale_validated.sale_price_value.clone()
        } else {
            self.current_floor.min(nft_for_sale_validated.sale_price_value.clone())
        };
        Self {
            nfts_for_sale: self.nfts_for_sale + 1,
            current_floor,
            ..self
        }
    }

    pub fn remove_sale(self, new_floor: Uint128) -> Self {
        Self {
            nfts_for_sale: self.nfts_for_sale - 1,
            current_floor: new_floor,
            ..self
        }
    }

    pub fn execute_sale(mut self, price_sold: Uint128, new_floor: Uint128) -> Self {
        self.nfts_for_sale -= 1;
        self.total_volume += price_sold;
        self.realized_trades += 1;
        self.current_floor = new_floor;
        self
    }
}

pub fn define_unique_collection_by_denom_id(nft_collection_address: &str, denom: &str) -> String {
    format!("{}{}", nft_collection_address, denom)
}

pub struct NftCollectionDenomIndexes<'a> {
    pub collection_index: MultiIndex<'a, Addr, NftCollectionInfoByDenom, String>,
}

impl IndexList<NftCollectionInfoByDenom> for NftCollectionDenomIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NftCollectionInfoByDenom>> + '_> {
        let v: Vec<&dyn Index<NftCollectionInfoByDenom>> = vec![&self.collection_index];
        Box::new(v.into_iter())
    }
}

pub fn nft_collection_denoms<'a>() -> IndexedMap<'a, String, NftCollectionInfoByDenom, NftCollectionDenomIndexes<'a>> {
    let indexes = NftCollectionDenomIndexes {
        collection_index: MultiIndex::new(
            |_, collection_denom| Addr::unchecked(collection_denom.nft_collection_address.clone()),
            "denoms",
            "denoms__nft_collection",
        )
    };
    IndexedMap::new("denoms", indexes)
}
