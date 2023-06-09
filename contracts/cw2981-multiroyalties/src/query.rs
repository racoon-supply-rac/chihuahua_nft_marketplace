use std::str::FromStr;

use cosmwasm_std::{Decimal, Deps, StdError, StdResult, Uint128};

use crate::msg::{CheckRoyaltiesResponse, RoyaltiesInfoResponse};
use crate::{Cw2981Contract, Royalty};

/// NOTE: default behaviour here is to round down
/// EIP2981 specifies that the rounding behaviour is at the discretion of the implementer
pub fn query_royalties_info(
    deps: Deps,
    token_id: String,
    sale_price: Uint128,
) -> StdResult<Vec<RoyaltiesInfoResponse>> {
    let contract = Cw2981Contract::default();
    let token_info = contract.tokens.load(deps.storage, &token_id)?;

    let mut royalties: Vec<Royalty> = vec![];
    if let Some(ref ext) = token_info.extension {
        royalties = ext.clone().royalties.unwrap_or_default();
    }

    // Make a vector of royalties
    if !royalties.is_empty() {
        let mut royalties_response: Vec<RoyaltiesInfoResponse> =
            Vec::with_capacity(royalties.len());
        for royalty in royalties.into_iter() {
            if Decimal::permille(royalty.royalty_permille_int) > Decimal::from_str("1.0").unwrap()
                || Decimal::permille(royalty.royalty_permille_int)
                    < Decimal::from_str("0.001").unwrap()
            {
                return Err(StdError::GenericErr {
                    msg: "InvalidRoyaltyValue".to_string(),
                });
            }
            let royalty_amount: Uint128 =
                sale_price * Decimal::permille(royalty.royalty_permille_int);
            royalties_response.push(RoyaltiesInfoResponse {
                address: royalty.receiver.to_string(),
                royalty_amount,
            })
        }
        Ok(royalties_response)
    } else {
        Ok(vec![])
    }
}

/// As our default implementation here specifies royalties at token level
/// and not at contract level, it is therefore logically true that
/// on sale, every token managed by this contract should be checked
/// to see if royalties are owed, and to whom. If you are importing
/// this logic, you may want a custom implementation here
pub fn check_royalties(_deps: Deps) -> StdResult<CheckRoyaltiesResponse> {
    Ok(CheckRoyaltiesResponse {
        royalty_payments: true,
    })
}
