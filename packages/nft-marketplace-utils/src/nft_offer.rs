use std::marker::PhantomData;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, Empty, MessageInfo, Timestamp, Uint128};
use cw721_base::helpers::Cw721Contract;
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};

use general_utils::denominations::{AcceptedDenominations, Denomination};
use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{
    CantOfferOnYourOwnNft, InvalidExpirationTimeForTheOffer, InvalidFundsForOffer,
    InvalidOfferDenom, InvalidOfferValueReceived, InvalidPrice, InvalidSellerInformation,
};

use crate::nft_collection::{NftCollectionAddress, TokenId};
use crate::nft_sale::define_unique_collection_nft_id;

#[cw_serde]
pub struct NftOffer {
    pub offerer_address: String,
    pub nft_collection_address: NftCollectionAddress,
    pub token_id: TokenId,
    pub offer_price_value: Uint128,
    pub offer_price_denom: Denomination,
    pub offer_expiration: Timestamp,
}

impl NftOffer {
    #[allow(clippy::too_many_arguments)]
    pub fn new_checked(
        deps: Deps,
        offer: NftOffer,
        info: &MessageInfo,
        current_time_seconds: u64,
        accepted_denominations: AcceptedDenominations,
        max_expiration_seconds: u64,
        min_expiration_seconds: u64,
        max_price: Uint128,
        min_price: Uint128,
    ) -> Result<Self, ContractError> {
        // Validate: Offerer, sender are identical and owner of the Token
        if info.sender != offer.offerer_address {
            return Err(ContractError::NftMarketplaceError(InvalidSellerInformation {}));
        }
        let owner_response = Cw721Contract::<Empty, Empty>(
            Addr::unchecked(offer.nft_collection_address.clone()),
            PhantomData,
            PhantomData,
        )
        .owner_of(&deps.querier, offer.token_id.clone(), false)?;

        if owner_response.owner == offer.offerer_address {
            return Err(ContractError::NftMarketplaceError(CantOfferOnYourOwnNft {}));
        }

        // Validate: The token ID is in the NFT collection
        Cw721Contract::<Empty, Empty>(
            Addr::unchecked(offer.nft_collection_address.clone()),
            PhantomData,
            PhantomData,
        )
        .nft_info::<String, Empty>(&deps.querier, offer.token_id.to_string())?;

        // Validate: Valid denom for the offer
        accepted_denominations.check_if_denom_is_accepted(&offer.offer_price_denom)?;

        // Validate: The funds and offer's denom and values
        if info.funds.len() != 1 {
            return Err(ContractError::NftMarketplaceError(InvalidFundsForOffer {}));
        }
        if info.funds[0].denom != offer.offer_price_denom {
            return Err(ContractError::NftMarketplaceError(InvalidOfferDenom {}));
        }
        if info.funds[0].amount != offer.offer_price_value {
            return Err(ContractError::NftMarketplaceError(InvalidOfferValueReceived {}));
        }
        if offer.offer_price_value > max_price || offer.offer_price_value < min_price {
            return Err(ContractError::NftMarketplaceError(InvalidPrice {}));
        }

        // Validate: Expiration of the offer
        let min_expiration = current_time_seconds + min_expiration_seconds;
        let max_expiration = current_time_seconds + max_expiration_seconds;

        if !(min_expiration < offer.offer_expiration.seconds()
            && offer.offer_expiration.seconds() <= max_expiration)
        {
            return Err(ContractError::NftMarketplaceError(
                InvalidExpirationTimeForTheOffer {},
            ));
        }

        Ok(NftOffer {
            offerer_address: offer.offerer_address,
            nft_collection_address: offer.nft_collection_address,
            token_id: offer.token_id,
            offer_price_value: offer.offer_price_value,
            offer_price_denom: offer.offer_price_denom,
            offer_expiration: offer.offer_expiration,
        })
    }
}

pub fn define_unique_offer(
    nft_collection_address: &NftCollectionAddress,
    token_id: &TokenId,
    offerer: &String,
) -> String {
    let collection_token_id = define_unique_collection_nft_id(nft_collection_address, token_id);
    format!("{}{}", collection_token_id, offerer)
}

pub struct NftOfferIndexes<'a> {
    pub collection_tokenid_index: MultiIndex<'a, String, NftOffer, String>,
    pub offerer_index: MultiIndex<'a, String, NftOffer, String>,
}

impl IndexList<NftOffer> for NftOfferIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NftOffer>> + '_> {
        let v: Vec<&dyn Index<NftOffer>> =
            vec![&self.collection_tokenid_index, &self.offerer_index];
        Box::new(v.into_iter())
    }
}

pub fn nft_offers<'a>() -> IndexedMap<'a, String, NftOffer, NftOfferIndexes<'a>> {
    let indexes = NftOfferIndexes {
        collection_tokenid_index: MultiIndex::new(
            |_, nft_offer| {
                define_unique_collection_nft_id(
                    &nft_offer.nft_collection_address,
                    &nft_offer.token_id,
                )
            },
            "offers",
            "offers__collection_tokenid",
        ),
        offerer_index: MultiIndex::new(
            |_, nft_offer| nft_offer.offerer_address.to_string(),
            "offers",
            "offers__offerer",
        ),
    };
    IndexedMap::new("offers", indexes)
}
