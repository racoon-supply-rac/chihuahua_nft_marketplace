use cosmwasm_std::{ensure, Deps, DepsMut, Env, MessageInfo, Response};

use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::{
    InvalidInput, InvalidNftCollection, NftCollectionAlreadyExists,
};
use nft_marketplace_utils::nft_collection::{
    define_unique_collection_by_denom_id, nft_collection_denoms, NftCollectionAddress,
    NftCollectionInfoAndUsdcVol, NftCollectionInfoByDenom, NftContractInfo,
};
use nft_marketplace_utils::response_handler::ResponseHandler;

use crate::helpers::royalties::validate_contract_type_and_royalty;
use crate::state::{CONFIG, GENERAL_STATS, LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL};

pub fn execute_add_new_nft_collection(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    nft_collection_address: NftCollectionAddress,
    nft_contract_info: NftContractInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !config
        .accepted_nft_code_ids
        .iter()
        .any(|contract_info| contract_info.equal(&nft_contract_info))
    {
        return Err(ContractError::NftMarketplaceError(InvalidNftCollection {}));
    }

    validate_input_and_if_already_exists(deps.as_ref(), &nft_collection_address)?;

    let new_info = save_new_coll_info(deps.branch(), &nft_collection_address, nft_contract_info)?;

    validate_contract_type_and_royalty(new_info, deps.querier)?;

    Ok(ResponseHandler::add_nft_collection(&nft_collection_address).response)
}

fn validate_input_and_if_already_exists(
    deps: Deps,
    nft_collection_address: &str,
) -> Result<(), ContractError> {
    // If it is a contract
    deps.querier
        .query_wasm_contract_info(nft_collection_address.to_string())
        .map_err(|_| ContractError::NftMarketplaceError(InvalidInput {}))?;

    // If it exists
    ensure!(
        LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL
            .may_load(deps.storage, nft_collection_address)?
            .is_none(),
        ContractError::NftMarketplaceError(NftCollectionAlreadyExists {})
    );

    Ok(())
}

fn save_new_coll_info(
    mut deps: DepsMut,
    nft_collection_address: &str,
    nft_contract_info: NftContractInfo,
) -> Result<NftCollectionInfoAndUsdcVol, ContractError> {
    create_nft_coll_denoms(deps.branch(), nft_collection_address.to_string())?;

    let new_info =
        NftCollectionInfoAndUsdcVol::new(nft_collection_address.to_string(), nft_contract_info);

    LISTED_NFT_COLLECTIONS_INFO_AND_USDC_VOL.save(
        deps.storage,
        nft_collection_address,
        &new_info,
    )?;

    // Update the last added collection
    GENERAL_STATS.update(deps.storage, |mut gen_stats| -> Result<_, ContractError> {
        gen_stats.last_collection_added = nft_collection_address.to_string();
        Ok(gen_stats)
    })?;

    Ok(new_info)
}

fn create_nft_coll_denoms(
    deps: DepsMut,
    nft_collection_address: NftCollectionAddress,
) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let collections = config
        .accepted_ibc_denominations
        .list_of_denoms
        .iter()
        .map(|accepted_denom| {
            NftCollectionInfoByDenom::new_checked(
                deps.querier,
                nft_collection_address.clone(),
                accepted_denom.clone(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    collections
        .into_iter()
        .try_for_each(|collection| {
            let collection_denom_unique =
                define_unique_collection_by_denom_id(&nft_collection_address, &collection.denom);
            nft_collection_denoms().save(deps.storage, collection_denom_unique, &collection)
        })
        .map_err(|_| ContractError::NftMarketplaceError(InvalidNftCollection {}))?;

    Ok(())
}
