#![allow(clippy::all)]
#![allow(warnings)]
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Empty};
use cw721_base::Cw721Contract;
pub use cw721_base::{ContractError, InstantiateMsg, MintMsg, MinterResponse};

pub use query::{check_royalties, query_royalties_info};

use crate::msg::Cw2981QueryMsg;

pub mod msg;
pub mod query;

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw2981-multiroyalties";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cw_serde]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[cw_serde]
pub struct Royalty {
    pub receiver: Addr,
    pub royalty_permille_int: u64,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    /// Vector of Royalties to be paid to different addresses
    pub royalties: Option<Vec<Royalty>>,
}

pub type Extension = Option<Metadata>;

pub type MintExtension = Option<Extension>;

pub type Cw2981Contract<'a> = Cw721Contract<'a, Extension, Empty, Empty, Cw2981QueryMsg>;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;
pub type QueryMsg = cw721_base::QueryMsg<Cw2981QueryMsg>;

#[cfg(not(feature = "library"))]
pub mod entry {
    use cosmwasm_std::{entry_point, StdError};
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    use super::*;

    #[entry_point]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        let res = Cw2981Contract::default().instantiate(deps.branch(), env, info, msg)?;
        // Explicitly set contract name and version, otherwise set to cw721-base info
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
            .map_err(ContractError::Std)?;
        Ok(res)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Mint(msg) => {
                if let Some(ext) = &msg.extension {
                    if let Some(royalties) = &ext.royalties {
                        let total_royalties: u64 =
                            royalties.iter().map(|r| r.royalty_permille_int).sum();
                        if total_royalties >= 1000 || total_royalties == 0 {
                            return Err(ContractError::Std(StdError::GenericErr {
                                msg: "Invalid Royalties".to_string(),
                            }));
                        }
                    }
                }
                Cw2981Contract::default().execute(
                    deps,
                    env,
                    info,
                    cw721_base::ExecuteMsg::Mint(msg),
                )
            }
            _ => Cw2981Contract::default().execute(deps, env, info, msg),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Extension { msg } => match msg {
                Cw2981QueryMsg::RoyaltyInfo {
                    token_id,
                    sale_price,
                } => cosmwasm_std::to_binary(&query_royalties_info(deps, token_id, sale_price)?),
                Cw2981QueryMsg::CheckRoyalties {} => {
                    cosmwasm_std::to_binary(&check_royalties(deps)?)
                }
            },
            _ => Cw2981Contract::default().query(deps, env, msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Uint128};
    use cw721::Cw721Query;

    use crate::msg::{CheckRoyaltiesResponse, RoyaltiesInfoResponse};

    use super::*;

    const CREATOR: &str = "creator";

    #[test]
    fn use_metadata_extension() {
        let mut deps = mock_dependencies();
        let contract = Cw2981Contract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: CREATOR.to_string(),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: "john".to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                ..Metadata::default()
            }),
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        entry::execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

        let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
        assert_eq!(res.token_uri, mint_msg.token_uri);
        assert_eq!(res.extension, mint_msg.extension);
    }

    #[test]
    fn check_royalties_response() {
        let mut deps = mock_dependencies();
        let _contract = Cw2981Contract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: CREATOR.to_string(),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: "john".to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                ..Metadata::default()
            }),
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg);
        entry::execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

        let expected = CheckRoyaltiesResponse {
            royalty_payments: true,
        };
        let res = check_royalties(deps.as_ref()).unwrap();
        assert_eq!(res, expected);

        // also check the longhand way
        let query_msg = QueryMsg::Extension {
            msg: Cw2981QueryMsg::CheckRoyalties {},
        };
        let query_res: CheckRoyaltiesResponse =
            from_binary(&entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(query_res, expected);
    }

    #[test]
    fn check_token_royalties() {
        let mut deps = mock_dependencies();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: CREATOR.to_string(),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: "jeanluc".to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                royalties: Some(vec![
                    Royalty {
                        receiver: Addr::unchecked("jeanluc".to_string()),
                        royalty_permille_int: 10,
                    },
                    Royalty {
                        receiver: Addr::unchecked("steve".to_string()),
                        royalty_permille_int: 20,
                    },
                ]),
                ..Metadata::default()
            }),
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        entry::execute(deps.as_mut(), mock_env(), info.clone(), exec_msg).unwrap();

        let expected = vec![
            RoyaltiesInfoResponse {
                address: mint_msg.owner,
                royalty_amount: Uint128::new(1),
            },
            RoyaltiesInfoResponse {
                address: "steve".to_string(),
                royalty_amount: Uint128::new(2),
            },
        ];
        let res =
            query_royalties_info(deps.as_ref(), token_id.to_string(), Uint128::new(100)).unwrap();
        assert_eq!(res, expected);

        // also check the longhand way
        let query_msg = QueryMsg::Extension {
            msg: Cw2981QueryMsg::RoyaltyInfo {
                token_id: token_id.to_string(),
                sale_price: Uint128::new(100),
            },
        };
        let query_res: Vec<RoyaltiesInfoResponse> =
            from_binary(&entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(query_res, expected);

        // check for rounding down
        // which is the default behaviour
        let voyager_token_id = "Voyager";
        let second_mint_msg = MintMsg {
            token_id: voyager_token_id.to_string(),
            owner: "janeway".to_string(),
            token_uri: Some("https://starships.example.com/Starship/Voyager.json".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Voyager".to_string()),
                royalties: Some(vec![
                    Royalty {
                        receiver: Addr::unchecked("janeway".to_string()),
                        royalty_permille_int: 20,
                    },
                    Royalty {
                        receiver: Addr::unchecked("mike".to_string()),
                        royalty_permille_int: 20,
                    },
                ]),
                ..Metadata::default()
            }),
        };
        let voyager_exec_msg = ExecuteMsg::Mint(second_mint_msg.clone());
        entry::execute(deps.as_mut(), mock_env(), info, voyager_exec_msg).unwrap();

        // 430 x 0.02 (i.e., 2%) is 8.6
        // we expect this to be rounded down to 8
        let voyager_expected = vec![
            RoyaltiesInfoResponse {
                address: second_mint_msg.owner,
                royalty_amount: Uint128::new(8),
            },
            RoyaltiesInfoResponse {
                address: "mike".to_string(),
                royalty_amount: Uint128::new(8),
            },
        ];

        let res = query_royalties_info(
            deps.as_ref(),
            voyager_token_id.to_string(),
            Uint128::new(430),
        )
        .unwrap();
        assert_eq!(res, voyager_expected);
    }
}
