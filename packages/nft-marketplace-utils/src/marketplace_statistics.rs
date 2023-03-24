use cosmwasm_std::{Decimal, Uint128};
use cosmwasm_schema::cw_serde;
use general_utils::denominations::Denomination;
use crate::nft_collection::NftCollectionAddress;
use crate::nft_sale::NftSale;

#[cw_serde]
pub struct CollectionVolume {
    pub nft_collection_address: NftCollectionAddress,
    pub usdc_volume: Uint128
}

#[cw_serde]
pub struct GeneralStats {
    pub last_collection_added: NftCollectionAddress,
    pub last_collections_traded: Vec<NftCollectionAddress>,
    pub top_10_volume_usdc: Vec<CollectionVolume>,
    pub lowest_volume_usdc: Uint128
}

impl GeneralStats {
    pub fn compute_new_top_10_and_latest_collection_traded(&mut self, new_collection_volume: CollectionVolume) -> &mut GeneralStats {
        // Top 10
        let mut new_top10 = self.top_10_volume_usdc.clone();
        if let Some(index) = new_top10.iter().position(|x| x.nft_collection_address == new_collection_volume.nft_collection_address) {
            new_top10.remove(index);
        }
        new_top10.push(new_collection_volume.clone());
        new_top10.sort_by_key(|v| v.usdc_volume);
        if new_top10.len() > 10 {
            new_top10.remove(0);
        }
        self.lowest_volume_usdc = new_top10.first().clone().unwrap().usdc_volume;
        self.top_10_volume_usdc = new_top10;

        // Last coll traded
        let mut new_last_traded = self.last_collections_traded.clone();
        if !new_last_traded.contains(&new_collection_volume.clone().nft_collection_address) {
            new_last_traded.push(new_collection_volume.clone().nft_collection_address);
        }
        if new_last_traded.len() > 10 {
            new_last_traded.remove(0);
        }
        self.last_collections_traded = new_last_traded;

        return self
    }
}

#[cw_serde]
pub struct MarketplaceStatsByDenom {
    pub denom: Denomination,
    pub nfts_for_sale: u64,
    pub realized_sales_counter: u64,
    pub total_realized_sales_volume: Uint128,
    pub total_marketplace_fees: Uint128,
    pub marketplace_fees_to_claim: Uint128
}

impl MarketplaceStatsByDenom {
    pub fn new(
        denom: Denomination
    ) -> Self {
        MarketplaceStatsByDenom {
            denom,
            nfts_for_sale: 0,
            realized_sales_counter: 0,
            total_realized_sales_volume: Uint128::zero(),
            total_marketplace_fees: Uint128::zero(),
            marketplace_fees_to_claim: Uint128::zero(),
        }
    }

    pub fn add_nfts_sold(
        &mut self,
        nb_sold: u64,
        total_value: Uint128
    ) -> &mut Self {
        self.realized_sales_counter += nb_sold.clone();
        self.nfts_for_sale -= nb_sold;
        self.total_realized_sales_volume += total_value;
        self
    }

    pub fn list_nft_for_sale(
        &mut self
    ) -> &mut Self {
        self.nfts_for_sale += 1;
        self
    }

    pub fn remove_nft_for_sale(
        &mut self
    ) -> &mut Self {
        self.nfts_for_sale -= 1;
        self
    }

    pub fn execute_sale(
        &mut self,
        sale_price: Uint128,
        marketplace_fees_pct: Decimal
    ) -> &mut Self {
        self.remove_nft_for_sale();
        self.total_realized_sales_volume += sale_price.clone();
        self.realized_sales_counter += 1;
        self.total_marketplace_fees += NftSale::compute_marketplace_fees(marketplace_fees_pct.clone(), sale_price.clone());
        self.marketplace_fees_to_claim += NftSale::compute_marketplace_fees(marketplace_fees_pct.clone(), sale_price.clone());
        self
    }
}