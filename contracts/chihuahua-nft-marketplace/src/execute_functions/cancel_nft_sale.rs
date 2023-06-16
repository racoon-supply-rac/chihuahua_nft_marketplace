use std::marker::PhantomData;

use cosmwasm_std::{ensure, Addr, DepsMut, Empty, Env, MessageInfo, Response};
use cw721_base::helpers::Cw721Contract;

use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{
    AdditionalInfoNeedsToBeFilled, CantUseAdditionalInfoIfNotContract,
    RevokeYourApprovalBeforeCancellingSale,
};
use nft_marketplace_utils::nft_collection::{
    define_unique_collection_by_denom_id, nft_collection_denoms, NftCollectionAddress, TokenId,
};
use nft_marketplace_utils::nft_sale::{
    compute_floor_collection_and_denom, define_unique_collection_nft_id, nfts_for_sale,
};
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::constants::MAX_PRICE;
use crate::state::MARKETPLACE_STATS_BY_DENOM;

pub fn execute_cancel_nft_sale(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    mut nft_collection_address: NftCollectionAddress,
    token_id: TokenId,
    additional_info: Option<String>,
) -> Result<Response, ContractError> {
    nft_collection_address = deps.api.addr_validate(&nft_collection_address)?.to_string();
    // Validation: additional_info can only be used by contract
    // When the contract uses this entry point it is for a cancellation for:
    // A. an update of a sale
    // B. Offer acceptation
    if let Some(addr) = additional_info.clone() {
        ensure!(
            info.sender == env.contract.address,
            ContractError::NftMarketplaceError(CantUseAdditionalInfoIfNotContract {})
        );
        info = MessageInfo {
            sender: deps.api.addr_validate(&addr)?,
            funds: info.funds.clone(),
        };
    } else {
        ensure!(
            info.sender != env.contract.address,
            ContractError::NftMarketplaceError(AdditionalInfoNeedsToBeFilled {})
        );
    }

    // Validate: Only the NFT owner can cancel a sale (or the contract for an offer change)
    // Validate: Sender is the owner
    let owner_response = Cw721Contract::<Empty, Empty>(
        deps.api.addr_validate(&nft_collection_address)?,
        PhantomData,
        PhantomData,
    )
    .owner_of(&deps.querier, token_id.clone(), false)?;
    let collection_token_id_unique =
        define_unique_collection_nft_id(&nft_collection_address, &token_id);

    let mut nft_for_sale_info =
        nfts_for_sale().load(deps.storage, collection_token_id_unique.clone())?;
    nft_for_sale_info = nft_for_sale_info.validate_sender_is_token_owner(
        info.sender.as_ref(),
        env.contract.address.as_ref(),
        &owner_response.owner,
    )?;

    // Validate: Need to revoke an approval before cancelling a Sale (otherwise a Transfer auto-cancels)
    if additional_info.is_none() {
        let cw721_contract = Cw721Contract::<Empty, Empty>(
            Addr::unchecked(nft_collection_address),
            PhantomData,
            PhantomData,
        );
        if cw721_contract
            .approval(
                &deps.querier,
                token_id,
                env.contract.address.to_string(),
                None,
            )
            .is_ok()
        {
            return Err(ContractError::NftMarketplaceError(
                RevokeYourApprovalBeforeCancellingSale {},
            ));
        }
    }

    // Update: Marketplace Statistics
    MARKETPLACE_STATS_BY_DENOM.update(
        deps.storage,
        &*nft_for_sale_info.sale_price_denom,
        |mp_info| -> Result<_, ContractError> {
            Ok(mp_info.unwrap().remove_nft_for_sale().clone())
        },
    )?;

    // Update: Remove the Sale
    let collection_denom_unique = define_unique_collection_by_denom_id(
        &nft_for_sale_info.nft_collection_address,
        &nft_for_sale_info.sale_price_denom,
    );

    nfts_for_sale().remove(deps.storage, collection_token_id_unique)?;

    // Execute: Compute new floor; if needed and update collection's stats (floor f.e.)
    let new_floor = compute_floor_collection_and_denom(
        deps.storage,
        nft_for_sale_info.sale_price_denom.clone(),
        nft_for_sale_info.nft_collection_address.clone(),
        MAX_PRICE,
    )?;

    nft_collection_denoms().update(
        deps.storage,
        collection_denom_unique,
        |nft_coll_denom| -> Result<_, ContractError> {
            Ok(nft_coll_denom.unwrap().remove_sale(new_floor))
        },
    )?;
    Ok(ResponseHandler::cancel_nft_sale_response(nft_for_sale_info).response)
}
