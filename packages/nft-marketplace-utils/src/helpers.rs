use std::marker::PhantomData;

use cosmwasm_std::{Deps, Empty};
use cw721_base::helpers::Cw721Contract;

use general_utils::error::ContractError;
use general_utils::error::NftMarketplaceError::YouDontOwnThisTokenID;

pub fn check_if_nft_is_owned(
    deps: Deps,
    sender: &str,
    nft_collection_address: &str,
    token_id: &str,
) -> Result<(), ContractError> {
    let owner_response = Cw721Contract::<Empty, Empty>(
        deps.api.addr_validate(nft_collection_address)?,
        PhantomData,
        PhantomData,
    )
    .owner_of(&deps.querier, token_id, false)?;
    if owner_response.owner != sender {
        return Err(ContractError::NftMarketplaceError(YouDontOwnThisTokenID {}));
    }
    Ok(())
}
