use cosmwasm_std::{DepsMut, Env, MessageInfo, Order, Response, StdResult};

use crate::state::CONFIG;
use general_utils::error::ContractError;
use nft_marketplace_utils::nft_sale::{nfts_for_sale, NftSale};

pub fn remove_expired_sales_function(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    for denom in config.accepted_ibc_denominations.list_of_denoms.iter() {
        let all_nfts_for_sale: Vec<(String, NftSale)> = nfts_for_sale()
            .idx
            .denom_index
            .prefix(denom.to_string())
            .range(deps.storage, None, None, Order::Ascending)
            .take(25)
            .collect::<StdResult<Vec<_>>>()?;
        for (unique_id, nft_sale) in all_nfts_for_sale.iter() {
            if nft_sale.sale_expiration < env.block.time {
                nfts_for_sale().remove(deps.storage, unique_id.clone())?;
            }
        }
    }

    Ok(Response::default())
}
