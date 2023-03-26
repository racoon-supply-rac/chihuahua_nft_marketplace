use std::marker::PhantomData;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Deps, Empty, MessageInfo, Order, StdResult, Storage, Timestamp, Uint128};
use cw721_base::helpers::Cw721Contract;
use cw_storage_plus::{Index, IndexedMap, IndexList, MultiIndex};
use crate::config::Config;
use general_utils::denominations::Denomination;
use general_utils::error::ContractError;
use general_utils::error::GenericError::InvalidFundsReceived;
use general_utils::error::NftMarketplaceError::{InvalidBuyerInformation, InvalidDenomOrValueReceivedForListingFee, InvalidExpirationTimeForTheSale, InvalidPriceForTheSale, InvalidSellerInformation, YouDontOwnThisTokenID};
use crate::inputs::Buyer;
use crate::nft_collection::{NftCollectionAddress, TokenId};

#[cw_serde]
pub struct NftSale {
    pub seller: String,
    pub nft_collection_address: NftCollectionAddress,
    pub token_id: TokenId,
    pub sale_price_value: Uint128,
    pub sale_price_denom: Denomination,
    pub sale_expiration: Timestamp
}

impl NftSale {
    pub fn new_checked(
        deps: Deps,
        current_time_seconds: &u64,
        info: &MessageInfo,
        sale_info: &NftSale,
        config: Config,
        contract_address: String,
        max_expiration_seconds: u64,
        min_expiration_seconds: u64,
        max_price: Uint128,
        min_price: Uint128
    ) -> Result<Self, ContractError> {
        // Validate: Received funds for listing fee
        if info.funds.len() != 1 {
            return Err(ContractError::Generic(InvalidFundsReceived {}));
        } else {
            if info.funds[0].denom != config.marketplace_listing_fee_denom ||
                info.funds[0].amount != config.marketplace_listing_fee_value
            {
                return Err(ContractError::NftMarketplace(InvalidDenomOrValueReceivedForListingFee {}));
            }
        }

        // Validate: Seller is Sender
        if info.sender != sale_info.seller {
            return Err(ContractError::NftMarketplace(InvalidSellerInformation {}))
        }

        // Validate: Token ID exists in the collection and the sender is the owner
        let owner_response = Cw721Contract::<Empty, Empty>(
            deps.api.addr_validate(&sale_info.nft_collection_address)?,
            PhantomData,
            PhantomData)
            .owner_of(
                &deps.querier,
                sale_info.token_id.clone().to_string(),
                false
            )?;
        if owner_response.owner != sale_info.seller {
            return Err(ContractError::NftMarketplace(YouDontOwnThisTokenID {}));
        }

        // Validate: If the contract can transfer the Token
        Cw721Contract::<Empty, Empty>(
            deps.api.addr_validate(&sale_info.nft_collection_address)?,
            PhantomData,
            PhantomData)
            .approval(
                &deps.querier,
                sale_info.token_id.clone(),
                contract_address.to_string(),
                None,
            )?;

        // Validate: If the denom for the sale is accepted
        config.accepted_ibc_denominations.check_if_denom_is_accepted( &sale_info.sale_price_denom.clone())?;

        // Validate: If the price is within bound
        if sale_info.sale_price_value > max_price || sale_info.sale_price_value < min_price {
            return Err(ContractError::NftMarketplace(InvalidPriceForTheSale {}));
        }

        // Validate: If the expiration is within bound
        let min_expiration = current_time_seconds + min_expiration_seconds;
        let max_expiration = current_time_seconds + max_expiration_seconds;

        if !(min_expiration < sale_info.sale_expiration.seconds() &&
            sale_info.sale_expiration.seconds() <= max_expiration) {
            return Err(ContractError::NftMarketplace(InvalidExpirationTimeForTheSale {}));
        }

        Ok(sale_info.clone())
    }

    pub fn validate_buying_information(self, buyer_info: &Buyer) -> Result<Self, ContractError> {
        let is_valid = buyer_info.denom == self.sale_price_denom
            && buyer_info.amount == self.sale_price_value
            && buyer_info.sender != self.seller;
        if is_valid {
            Ok(self)
        } else {
            Err(ContractError::NftMarketplace(InvalidBuyerInformation {}))
        }
    }

    pub fn compute_marketplace_fees(marketplace_fees_pct: Decimal, sale_value: Uint128) -> Uint128 {
        return sale_value * marketplace_fees_pct;
    }

    pub fn validate_sender_is_token_owner(
        self,
        sender_address: &str,
        contract_addr: &str,
        token_owner: &str
    ) -> Result<Self, ContractError> {
        if sender_address == contract_addr || token_owner == sender_address {
            return Ok(self)
        } else {
            Err(ContractError::NftMarketplace(YouDontOwnThisTokenID {}))
        }
    }
}

pub fn define_unique_collection_nft_id(nft_collection_address: &NftCollectionAddress, token_id: &TokenId) -> String {
    let mut unique_index: String = nft_collection_address.to_string().to_owned();
    unique_index.push_str(&*token_id);
    unique_index
}

pub struct NftCollectionSaleIndexes<'a> {
    pub collection_index: MultiIndex<'a, String, NftSale, String>,
    pub seller_index: MultiIndex<'a, String, NftSale, String>,
    pub denom_index: MultiIndex<'a, String, NftSale, String>
}

impl IndexList<NftSale> for NftCollectionSaleIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NftSale>> + '_> {
        let v: Vec<&dyn Index<NftSale>> = vec![&self.collection_index, &self.seller_index, &self.denom_index];
        Box::new(v.into_iter())
    }
}

pub fn nfts_for_sale<'a>() -> IndexedMap<'a, String, NftSale, NftCollectionSaleIndexes<'a>> {
    let indexes = NftCollectionSaleIndexes {
        collection_index: MultiIndex::new(
            |_, nft_sale| nft_sale.nft_collection_address.clone(),
            "sales",
            "sales__collection",
        ),
        seller_index: MultiIndex::new(
            |_, nft_sale| nft_sale.seller.clone(),
            "sales",
            "sales__seller",
        ),
        denom_index: MultiIndex::new(
            |_, nft_sale| nft_sale.sale_price_denom.clone(),
            "sales",
            "sales__denom",
        )
    };
    IndexedMap::new("sales", indexes)
}

pub fn compute_floor_collection_and_denom(store: &mut dyn Storage, denom: String,
                                          nft_collection_address: NftCollectionAddress,
                                          max_price: Uint128) -> StdResult<Uint128> {
    let mut nfts_for_sale_info: Vec<Uint128> = Vec::new();
    for result in nfts_for_sale()
        .idx
        .denom_index
        .prefix(denom)
        .range(store, None, None, Order::Ascending)
    {
        match result {
            Err(e) => return Err(e.into()),
            Ok((_, sale_info)) => {
                if sale_info.nft_collection_address == nft_collection_address {
                    nfts_for_sale_info.push(sale_info.sale_price_value);
                }
            }
        }
    }
    let result = nfts_for_sale_info.iter().min().unwrap_or(&max_price).clone();

    Ok(result)
}

pub fn save_nfts_for_sale(store: &mut dyn Storage, nft_for_sale: &NftSale) -> StdResult<()> {
    nfts_for_sale().save(store, nft_for_sale.token_id.clone(), nft_for_sale)
}

#[cw_serde]
pub struct TokenSaleHistory {
    pub seller: String,
    pub buyer: String,
    pub nft_collection_address: NftCollectionAddress,
    pub token_id: TokenId,
    pub sale_price_value: Uint128,
    pub sale_price_denom: String,
    pub sale_time: Timestamp
}