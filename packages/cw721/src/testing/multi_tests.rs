use crate::{
    error::Cw721ContractError,
    extension::Cw721OnchainExtensions,
    msg::{
        Cw721ExecuteMsg, Cw721InstantiateMsg, Cw721MigrateMsg, Cw721QueryMsg, MinterResponse,
        NumTokensResponse, OwnerOfResponse,
    },
    state::{NftExtension, Trait},
    traits::{Cw721Execute, Cw721Query},
    DefaultOptionalCollectionExtension, DefaultOptionalCollectionExtensionMsg,
    DefaultOptionalNftExtension, DefaultOptionalNftExtensionMsg, NftExtensionMsg,
};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, QuerierWrapper, Response,
    WasmMsg,
};
use cw721_016::NftInfoResponse;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use cw_ownable::{Ownership, OwnershipError};
use cw_utils::Expiration;
use url::ParseError;

pub const CREATOR_ADDR: &str = "creator";
pub const MINTER_ADDR: &str = "minter";
pub const OTHER_ADDR: &str = "other";
pub const NFT_OWNER_ADDR: &str = "nft_owner";

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721InstantiateMsg<DefaultOptionalCollectionExtensionMsg>,
) -> Result<Response, Cw721ContractError> {
    let contract = Cw721OnchainExtensions::default();
    contract.instantiate_with_version(deps, &env, &info, msg, "contract_name", "contract_version")
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721ExecuteMsg<
        DefaultOptionalNftExtensionMsg,
        DefaultOptionalCollectionExtensionMsg,
        Empty,
    >,
) -> Result<Response, Cw721ContractError> {
    let contract = Cw721OnchainExtensions::default();
    contract.execute(deps, &env, &info, msg)
}

pub fn query(
    deps: Deps,
    env: Env,
    msg: Cw721QueryMsg<DefaultOptionalNftExtension, DefaultOptionalCollectionExtension, Empty>,
) -> Result<Binary, Cw721ContractError> {
    let contract = Cw721OnchainExtensions::default();
    contract.query(deps, &env, msg)
}

pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: Cw721MigrateMsg,
) -> Result<Response, Cw721ContractError> {
    let contract = Cw721OnchainExtensions::default();
    contract.migrate(deps, env, msg, "contract_name", "contract_version")
}

fn cw721_base_latest_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query).with_migrate(migrate);
    Box::new(contract)
}

fn cw721_base_015_contract() -> Box<dyn Contract<Empty>> {
    use cw721_base_015 as v15;
    let contract = ContractWrapper::new(
        v15::entry::execute,
        v15::entry::instantiate,
        v15::entry::query,
    );
    Box::new(contract)
}

fn cw721_base_016_contract() -> Box<dyn Contract<Empty>> {
    use cw721_base_016 as v16;
    let contract = ContractWrapper::new(
        v16::entry::execute,
        v16::entry::instantiate,
        v16::entry::query,
    );
    Box::new(contract)
}

fn cw721_base_017_contract() -> Box<dyn Contract<Empty>> {
    use cw721_base_017 as v17;
    let contract = ContractWrapper::new(
        v17::entry::execute,
        v17::entry::instantiate,
        v17::entry::query,
    );
    Box::new(contract)
}

fn cw721_base_018_contract() -> Box<dyn Contract<Empty>> {
    use cw721_base_018 as v18;
    let contract = ContractWrapper::new(
        v18::entry::execute,
        v18::entry::instantiate,
        v18::entry::query,
    );
    Box::new(contract)
}

fn query_owner(querier: QuerierWrapper, cw721: &Addr, token_id: String) -> Addr {
    let resp: OwnerOfResponse = querier
        .query_wasm_smart(
            cw721,
            &Cw721QueryMsg::<
                DefaultOptionalNftExtension,
                DefaultOptionalCollectionExtension,
                Empty,
            >::OwnerOf {
                token_id,
                include_expired: None,
            },
        )
        .unwrap();
    Addr::unchecked(resp.owner)
}

fn query_nft_info(
    querier: QuerierWrapper,
    cw721: &Addr,
    token_id: String,
) -> NftInfoResponse<Option<NftExtension>> {
    querier
        .query_wasm_smart(
            cw721,
            &Cw721QueryMsg::<
                DefaultOptionalNftExtension,
                DefaultOptionalCollectionExtension,
                Empty,
            >::NftInfo {
                token_id,
            },
        )
        .unwrap()
}

fn mint_transfer_and_burn(app: &mut App, cw721: Addr, sender: Addr, token_id: String) {
    app.execute_contract(
        sender.clone(),
        cw721.clone(),
        &Cw721ExecuteMsg::<Empty, Empty, Empty>::Mint {
            token_id: token_id.clone(),
            owner: sender.to_string(),
            token_uri: None,
            extension: Empty::default(),
        },
        &[],
    )
    .unwrap();

    let owner = query_owner(app.wrap(), &cw721, token_id.clone());
    assert_eq!(owner, sender.to_string());

    app.execute_contract(
        sender,
        cw721.clone(),
        &Cw721ExecuteMsg::<Empty, Empty, Empty>::TransferNft {
            recipient: "burner".to_string(),
            token_id: token_id.clone(),
        },
        &[],
    )
    .unwrap();

    let owner = query_owner(app.wrap(), &cw721, token_id.clone());
    assert_eq!(owner, "burner".to_string());

    app.execute_contract(
        Addr::unchecked("burner"),
        cw721,
        &Cw721ExecuteMsg::<Empty, Empty, Empty>::Burn { token_id },
        &[],
    )
    .unwrap();
}

#[test]
fn test_operator() {
    // --- setup ---
    let mut app = App::default();
    let admin = Addr::unchecked("admin");
    let code_id = app.store_code(cw721_base_latest_contract());
    let other = Addr::unchecked(OTHER_ADDR);
    let cw721 = app
        .instantiate_contract(
            code_id,
            other.clone(),
            &Cw721InstantiateMsg::<DefaultOptionalCollectionExtension> {
                name: "collection".to_string(),
                symbol: "symbol".to_string(),
                minter: Some(MINTER_ADDR.to_string()),
                creator: Some(CREATOR_ADDR.to_string()),
                collection_info_extension: None,
                withdraw_address: None,
            },
            &[],
            "cw721-base",
            Some(admin.to_string()),
        )
        .unwrap();
    // mint
    let minter = Addr::unchecked(MINTER_ADDR);
    let nft_owner = Addr::unchecked(NFT_OWNER_ADDR);
    app.execute_contract(
        minter,
        cw721.clone(),
        &Cw721ExecuteMsg::<Empty, Empty, Empty>::Mint {
            token_id: "1".to_string(),
            owner: nft_owner.to_string(),
            token_uri: Some("".to_string()), // empty uri, response contains attribute with value "empty"
            extension: Empty::default(),
        },
        &[],
    )
    .unwrap();

    // --- test operator/approve all ---
    // owner adds other user as operator using approve all
    app.execute_contract(
        nft_owner.clone(),
        cw721.clone(),
        &Cw721ExecuteMsg::<Empty, Empty, Empty>::ApproveAll {
            operator: other.to_string(),
            expires: Some(Expiration::Never {}),
        },
        &[],
    )
    .unwrap();

    // transfer by operator
    app.execute_contract(
        other.clone(),
        cw721.clone(),
        &Cw721ExecuteMsg::<Empty, Empty, Empty>::TransferNft {
            recipient: other.to_string(),
            token_id: "1".to_string(),
        },
        &[],
    )
    .unwrap();
    // check other is new owner
    let owner_response: OwnerOfResponse = app
        .wrap()
        .query_wasm_smart(
            &cw721,
            &Cw721QueryMsg::<
                DefaultOptionalNftExtension,
                DefaultOptionalCollectionExtension,
                Empty,
            >::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            },
        )
        .unwrap();
    assert_eq!(owner_response.owner, other.to_string());
    // check previous owner cant transfer
    let err: Cw721ContractError = app
        .execute_contract(
            nft_owner.clone(),
            cw721.clone(),
            &Cw721ExecuteMsg::<Empty, Empty, Empty>::TransferNft {
                recipient: other.to_string(),
                token_id: "1".to_string(),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, Cw721ContractError::Ownership(OwnershipError::NotOwner));

    // transfer back to previous owner
    app.execute_contract(
        other.clone(),
        cw721.clone(),
        &Cw721ExecuteMsg::<Empty, Empty, Empty>::TransferNft {
            recipient: nft_owner.to_string(),
            token_id: "1".to_string(),
        },
        &[],
    )
    .unwrap();
    // check owner
    let owner_response: OwnerOfResponse = app
        .wrap()
        .query_wasm_smart(
            &cw721,
            &Cw721QueryMsg::<
                DefaultOptionalNftExtension,
                DefaultOptionalCollectionExtension,
                Empty,
            >::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            },
        )
        .unwrap();
    assert_eq!(owner_response.owner, nft_owner.to_string());

    // other user is still operator and can transfer!
    app.execute_contract(
        other.clone(),
        cw721.clone(),
        &Cw721ExecuteMsg::<Empty, Empty, Empty>::TransferNft {
            recipient: other.to_string(),
            token_id: "1".to_string(),
        },
        &[],
    )
    .unwrap();
    // check other is new owner
    let owner_response: OwnerOfResponse = app
        .wrap()
        .query_wasm_smart(
            &cw721,
            &Cw721QueryMsg::<
                DefaultOptionalNftExtension,
                DefaultOptionalCollectionExtension,
                Empty,
            >::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            },
        )
        .unwrap();
    assert_eq!(owner_response.owner, other.to_string());

    // -- test revoke
    // transfer to previous owner
    app.execute_contract(
        other.clone(),
        cw721.clone(),
        &Cw721ExecuteMsg::<Empty, Empty, Empty>::TransferNft {
            recipient: nft_owner.to_string(),
            token_id: "1".to_string(),
        },
        &[],
    )
    .unwrap();

    // revoke operator
    app.execute_contract(
        nft_owner,
        cw721.clone(),
        &Cw721ExecuteMsg::<Empty, Empty, Empty>::RevokeAll {
            operator: other.to_string(),
        },
        &[],
    )
    .unwrap();

    // other not operator anymore and cant send
    let err: Cw721ContractError = app
        .execute_contract(
            other.clone(),
            cw721,
            &Cw721ExecuteMsg::<Empty, Empty, Empty>::TransferNft {
                recipient: other.to_string(),
                token_id: "1".to_string(),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, Cw721ContractError::Ownership(OwnershipError::NotOwner));
}

/// Instantiates a 0.16 version of this contract and tests that tokens
/// can be minted, transferred, and burnred after migration.
#[test]
fn test_migration_legacy_to_latest() {
    // v0.15 migration by using existing minter addr
    {
        use cw721_base_015 as v15;
        let mut app = App::default();
        let admin = Addr::unchecked("admin");

        let code_id_016 = app.store_code(cw721_base_015_contract());
        let code_id_latest = app.store_code(cw721_base_latest_contract());

        let legacy_creator_and_minter = Addr::unchecked("legacy_creator_and_minter");

        let cw721 = app
            .instantiate_contract(
                code_id_016,
                legacy_creator_and_minter.clone(),
                &v15::InstantiateMsg {
                    name: "collection".to_string(),
                    symbol: "symbol".to_string(),
                    minter: legacy_creator_and_minter.to_string(),
                },
                &[],
                "cw721-base",
                Some(admin.to_string()),
            )
            .unwrap();

        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // migrate
        app.execute(
            admin,
            WasmMsg::Migrate {
                contract_addr: cw721.to_string(),
                new_code_id: code_id_latest,
                msg: to_json_binary(&Cw721MigrateMsg::WithUpdate {
                    minter: None,
                    creator: None,
                })
                .unwrap(),
            }
            .into(),
        )
        .unwrap();

        // non-minter user cant mint
        let other = Addr::unchecked(OTHER_ADDR);
        let err: Cw721ContractError = app
            .execute_contract(
                other.clone(),
                cw721.clone(),
                &Cw721ExecuteMsg::<Empty, Empty, Empty>::Mint {
                    token_id: "1".to_string(),
                    owner: other.to_string(),
                    token_uri: Some("ipfs://new.uri".to_string()),
                    extension: Empty::default(),
                },
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(err, Cw721ContractError::NotMinter {});

        // legacy minter can still mint
        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // check new mint query response works.
        #[allow(deprecated)]
        let m: MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(legacy_creator_and_minter.to_string()));

        // check that the new response is backwards compatable when minter
        // is not None.
        #[allow(deprecated)]
        let m: v15::MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, legacy_creator_and_minter.to_string());

        // check minter ownership query works
        let minter_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetMinterOwnership {},
            )
            .unwrap();
        assert_eq!(
            minter_ownership.owner,
            Some(legacy_creator_and_minter.clone())
        );

        // check creator ownership query works
        let creator_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetCreatorOwnership {},
            )
            .unwrap();
        assert_eq!(creator_ownership.owner, Some(legacy_creator_and_minter));
    }
    // v0.15 migration by providing new creator and minter addr
    {
        use cw721_base_015 as v15;
        let mut app = App::default();
        let admin = Addr::unchecked("admin");

        let code_id_015 = app.store_code(cw721_base_015_contract());
        let code_id_latest = app.store_code(cw721_base_latest_contract());

        let legacy_creator_and_minter = Addr::unchecked("legacy_creator_and_minter");

        let cw721 = app
            .instantiate_contract(
                code_id_015,
                legacy_creator_and_minter.clone(),
                &v15::InstantiateMsg {
                    name: "collection".to_string(),
                    symbol: "symbol".to_string(),
                    minter: legacy_creator_and_minter.to_string(),
                },
                &[],
                "cw721-base",
                Some(admin.to_string()),
            )
            .unwrap();

        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // migrate
        app.execute(
            admin,
            WasmMsg::Migrate {
                contract_addr: cw721.to_string(),
                new_code_id: code_id_latest,
                msg: to_json_binary(&Cw721MigrateMsg::WithUpdate {
                    minter: Some(MINTER_ADDR.to_string()),
                    creator: Some(CREATOR_ADDR.to_string()),
                })
                .unwrap(),
            }
            .into(),
        )
        .unwrap();

        // legacy minter user cant mint
        let err: Cw721ContractError = app
            .execute_contract(
                legacy_creator_and_minter.clone(),
                cw721.clone(),
                &Cw721ExecuteMsg::<Empty, Empty, Empty>::Mint {
                    token_id: "1".to_string(),
                    owner: legacy_creator_and_minter.to_string(),
                    token_uri: Some("ipfs://new.uri".to_string()),
                    extension: Empty::default(),
                },
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(err, Cw721ContractError::NotMinter {});

        // new minter can mint
        let minter = Addr::unchecked(MINTER_ADDR);
        mint_transfer_and_burn(&mut app, cw721.clone(), minter.clone(), "1".to_string());

        // check new mint query response works.
        #[allow(deprecated)]
        let m: MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(minter.to_string()));

        // check that the new response is backwards compatable when minter
        // is not None.
        #[allow(deprecated)]
        let m: v15::MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, minter.to_string());

        // check minter ownership query works
        let minter_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetMinterOwnership {},
            )
            .unwrap();
        assert_eq!(minter_ownership.owner, Some(minter));

        // check creator ownership query works
        let creator = Addr::unchecked(CREATOR_ADDR);
        let creator_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetCreatorOwnership {},
            )
            .unwrap();
        assert_eq!(creator_ownership.owner, Some(creator));
    }
    // v0.16 migration by using existing minter addr
    {
        use cw721_base_016 as v16;
        let mut app = App::default();
        let admin = Addr::unchecked("admin");

        let code_id_016 = app.store_code(cw721_base_016_contract());
        let code_id_latest = app.store_code(cw721_base_latest_contract());

        let legacy_creator_and_minter = Addr::unchecked("legacy_creator_and_minter");

        let cw721 = app
            .instantiate_contract(
                code_id_016,
                legacy_creator_and_minter.clone(),
                &v16::InstantiateMsg {
                    name: "collection".to_string(),
                    symbol: "symbol".to_string(),
                    minter: legacy_creator_and_minter.to_string(),
                },
                &[],
                "cw721-base",
                Some(admin.to_string()),
            )
            .unwrap();

        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // migrate
        app.execute(
            admin,
            WasmMsg::Migrate {
                contract_addr: cw721.to_string(),
                new_code_id: code_id_latest,
                msg: to_json_binary(&Cw721MigrateMsg::WithUpdate {
                    minter: None,
                    creator: None,
                })
                .unwrap(),
            }
            .into(),
        )
        .unwrap();

        // non-minter user cant mint
        let other = Addr::unchecked(OTHER_ADDR);
        let err: Cw721ContractError = app
            .execute_contract(
                other.clone(),
                cw721.clone(),
                &Cw721ExecuteMsg::<Empty, Empty, Empty>::Mint {
                    token_id: "1".to_string(),
                    owner: other.to_string(),
                    token_uri: Some("ipfs://new.uri".to_string()),
                    extension: Empty::default(),
                },
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(err, Cw721ContractError::NotMinter {});

        // legacy minter can still mint
        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // check new mint query response works.
        #[allow(deprecated)]
        let m: MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(legacy_creator_and_minter.to_string()));

        // check that the new response is backwards compatable when minter
        // is not None.
        #[allow(deprecated)]
        let m: v16::MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, legacy_creator_and_minter.to_string());

        // check minter ownership query works
        let minter_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetMinterOwnership {},
            )
            .unwrap();
        assert_eq!(
            minter_ownership.owner,
            Some(legacy_creator_and_minter.clone())
        );

        // check creator ownership query works
        let creator_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetCreatorOwnership {},
            )
            .unwrap();
        assert_eq!(creator_ownership.owner, Some(legacy_creator_and_minter));
    }
    // v0.16 migration by providing new creator and minter addr
    {
        use cw721_base_016 as v16;
        let mut app = App::default();
        let admin = Addr::unchecked("admin");

        let code_id_016 = app.store_code(cw721_base_016_contract());
        let code_id_latest = app.store_code(cw721_base_latest_contract());

        let legacy_creator_and_minter = Addr::unchecked("legacy_creator_and_minter");

        let cw721 = app
            .instantiate_contract(
                code_id_016,
                legacy_creator_and_minter.clone(),
                &v16::InstantiateMsg {
                    name: "collection".to_string(),
                    symbol: "symbol".to_string(),
                    minter: legacy_creator_and_minter.to_string(),
                },
                &[],
                "cw721-base",
                Some(admin.to_string()),
            )
            .unwrap();

        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // migrate
        app.execute(
            admin,
            WasmMsg::Migrate {
                contract_addr: cw721.to_string(),
                new_code_id: code_id_latest,
                msg: to_json_binary(&Cw721MigrateMsg::WithUpdate {
                    minter: Some(MINTER_ADDR.to_string()),
                    creator: Some(CREATOR_ADDR.to_string()),
                })
                .unwrap(),
            }
            .into(),
        )
        .unwrap();

        // legacy minter user cant mint
        let err: Cw721ContractError = app
            .execute_contract(
                legacy_creator_and_minter.clone(),
                cw721.clone(),
                &Cw721ExecuteMsg::<Empty, Empty, Empty>::Mint {
                    token_id: "1".to_string(),
                    owner: legacy_creator_and_minter.to_string(),
                    token_uri: Some("ipfs://new.uri".to_string()),
                    extension: Empty::default(),
                },
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(err, Cw721ContractError::NotMinter {});

        // new minter can mint
        let minter = Addr::unchecked(MINTER_ADDR);
        mint_transfer_and_burn(&mut app, cw721.clone(), minter.clone(), "1".to_string());

        // check new mint query response works.
        #[allow(deprecated)]
        let m: MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(minter.to_string()));

        // check that the new response is backwards compatable when minter
        // is not None.
        #[allow(deprecated)]
        let m: v16::MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, minter.to_string());

        // check minter ownership query works
        let minter_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetMinterOwnership {},
            )
            .unwrap();
        assert_eq!(minter_ownership.owner, Some(minter));

        // check creator ownership query works
        let creator = Addr::unchecked(CREATOR_ADDR);
        let creator_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetCreatorOwnership {},
            )
            .unwrap();
        assert_eq!(creator_ownership.owner, Some(creator));
    }
    // v0.17 migration by using existing minter addr
    {
        use cw721_base_017 as v17;
        let mut app = App::default();
        let admin = Addr::unchecked("admin");

        let code_id_017 = app.store_code(cw721_base_017_contract());
        let code_id_latest = app.store_code(cw721_base_latest_contract());

        let legacy_creator_and_minter = Addr::unchecked("legacy_creator_and_minter");

        let cw721 = app
            .instantiate_contract(
                code_id_017,
                legacy_creator_and_minter.clone(),
                &v17::InstantiateMsg {
                    name: "collection".to_string(),
                    symbol: "symbol".to_string(),
                    minter: legacy_creator_and_minter.to_string(),
                },
                &[],
                "cw721-base",
                Some(admin.to_string()),
            )
            .unwrap();

        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // migrate
        app.execute(
            admin,
            WasmMsg::Migrate {
                contract_addr: cw721.to_string(),
                new_code_id: code_id_latest,
                msg: to_json_binary(&Cw721MigrateMsg::WithUpdate {
                    minter: None,
                    creator: None,
                })
                .unwrap(),
            }
            .into(),
        )
        .unwrap();

        // non-minter user cant mint
        let other = Addr::unchecked(OTHER_ADDR);
        let err: Cw721ContractError = app
            .execute_contract(
                other.clone(),
                cw721.clone(),
                &Cw721ExecuteMsg::<Empty, Empty, Empty>::Mint {
                    token_id: "1".to_string(),
                    owner: other.to_string(),
                    token_uri: Some("ipfs://new.uri".to_string()),
                    extension: Empty::default(),
                },
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(err, Cw721ContractError::NotMinter {});

        // legacy minter can still mint
        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // check new mint query response works.
        #[allow(deprecated)]
        let m: MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(legacy_creator_and_minter.to_string()));

        // check that the new response is backwards compatable when minter
        // is not None.
        #[allow(deprecated)]
        let m: v17::MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(legacy_creator_and_minter.to_string()));

        // check minter ownership query works
        let minter_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetMinterOwnership {},
            )
            .unwrap();
        assert_eq!(
            minter_ownership.owner,
            Some(legacy_creator_and_minter.clone())
        );

        // check creator ownership query works
        let creator_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetCreatorOwnership {},
            )
            .unwrap();
        assert_eq!(creator_ownership.owner, Some(legacy_creator_and_minter));
    }
    // v0.17 migration by providing new creator and minter addr
    {
        use cw721_base_017 as v17;
        let mut app = App::default();
        let admin = Addr::unchecked("admin");

        let code_id_017 = app.store_code(cw721_base_017_contract());
        let code_id_latest = app.store_code(cw721_base_latest_contract());

        let legacy_creator_and_minter = Addr::unchecked("legacy_creator_and_minter");

        let cw721 = app
            .instantiate_contract(
                code_id_017,
                legacy_creator_and_minter.clone(),
                &v17::InstantiateMsg {
                    name: "collection".to_string(),
                    symbol: "symbol".to_string(),
                    minter: legacy_creator_and_minter.to_string(),
                },
                &[],
                "cw721-base",
                Some(admin.to_string()),
            )
            .unwrap();

        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // migrate
        app.execute(
            admin,
            WasmMsg::Migrate {
                contract_addr: cw721.to_string(),
                new_code_id: code_id_latest,
                msg: to_json_binary(&Cw721MigrateMsg::WithUpdate {
                    minter: Some(MINTER_ADDR.to_string()),
                    creator: Some(CREATOR_ADDR.to_string()),
                })
                .unwrap(),
            }
            .into(),
        )
        .unwrap();

        // legacy minter user cant mint
        let err: Cw721ContractError = app
            .execute_contract(
                legacy_creator_and_minter.clone(),
                cw721.clone(),
                &Cw721ExecuteMsg::<Empty, Empty, Empty>::Mint {
                    token_id: "1".to_string(),
                    owner: legacy_creator_and_minter.to_string(),
                    token_uri: Some("ipfs://new.uri".to_string()),
                    extension: Empty::default(),
                },
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(err, Cw721ContractError::NotMinter {});

        // new minter can mint
        let minter = Addr::unchecked(MINTER_ADDR);
        mint_transfer_and_burn(&mut app, cw721.clone(), minter.clone(), "1".to_string());

        // check new mint query response works.
        #[allow(deprecated)]
        let m: MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(minter.to_string()));

        // check that the new response is backwards compatable when minter
        // is not None.
        #[allow(deprecated)]
        let m: v17::MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(minter.to_string()));

        // check minter ownership query works
        let minter_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetMinterOwnership {},
            )
            .unwrap();
        assert_eq!(minter_ownership.owner, Some(minter));

        // check creator ownership query works
        let creator = Addr::unchecked(CREATOR_ADDR);
        let creator_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetCreatorOwnership {},
            )
            .unwrap();
        assert_eq!(creator_ownership.owner, Some(creator));
    }
    // v0.18 migration by using existing minter addr
    {
        use cw721_base_018 as v18;
        let mut app = App::default();
        let admin = Addr::unchecked("admin");

        let code_id_018 = app.store_code(cw721_base_018_contract());
        let code_id_latest = app.store_code(cw721_base_latest_contract());

        let legacy_creator_and_minter = Addr::unchecked("legacy_creator_and_minter");

        let cw721 = app
            .instantiate_contract(
                code_id_018,
                legacy_creator_and_minter.clone(),
                &v18::InstantiateMsg {
                    name: "collection".to_string(),
                    symbol: "symbol".to_string(),
                    minter: legacy_creator_and_minter.to_string(),
                },
                &[],
                "cw721-base",
                Some(admin.to_string()),
            )
            .unwrap();

        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // migrate
        app.execute(
            admin,
            WasmMsg::Migrate {
                contract_addr: cw721.to_string(),
                new_code_id: code_id_latest,
                msg: to_json_binary(&Cw721MigrateMsg::WithUpdate {
                    minter: None,
                    creator: None,
                })
                .unwrap(),
            }
            .into(),
        )
        .unwrap();

        // non-minter user cant mint
        let other = Addr::unchecked(OTHER_ADDR);
        let err: Cw721ContractError = app
            .execute_contract(
                other.clone(),
                cw721.clone(),
                &Cw721ExecuteMsg::<Empty, Empty, Empty>::Mint {
                    token_id: "1".to_string(),
                    owner: other.to_string(),
                    token_uri: Some("ipfs://new.uri".to_string()),
                    extension: Empty::default(),
                },
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(err, Cw721ContractError::NotMinter {});

        // legacy minter can still mint
        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // check new mint query response works.
        #[allow(deprecated)]
        let m: MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(legacy_creator_and_minter.to_string()));

        // check that the new response is backwards compatable when minter
        // is not None.
        #[allow(deprecated)]
        let m: v18::MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(legacy_creator_and_minter.to_string()));

        // check minter ownership query works
        let minter_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetMinterOwnership {},
            )
            .unwrap();
        assert_eq!(
            minter_ownership.owner,
            Some(legacy_creator_and_minter.clone())
        );

        // check creator ownership query works
        let creator_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetCreatorOwnership {},
            )
            .unwrap();
        assert_eq!(creator_ownership.owner, Some(legacy_creator_and_minter));
    }
    // v0.18 migration by providing new creator and minter addr
    {
        use cw721_base_018 as v18;
        let mut app = App::default();
        let admin = Addr::unchecked("admin");

        let code_id_018 = app.store_code(cw721_base_018_contract());
        let code_id_latest = app.store_code(cw721_base_latest_contract());

        let legacy_creator_and_minter = Addr::unchecked("legacy_creator_and_minter");

        let cw721 = app
            .instantiate_contract(
                code_id_018,
                legacy_creator_and_minter.clone(),
                &v18::InstantiateMsg {
                    name: "collection".to_string(),
                    symbol: "symbol".to_string(),
                    minter: legacy_creator_and_minter.to_string(),
                },
                &[],
                "cw721-base",
                Some(admin.to_string()),
            )
            .unwrap();

        mint_transfer_and_burn(
            &mut app,
            cw721.clone(),
            legacy_creator_and_minter.clone(),
            "1".to_string(),
        );

        // migrate
        app.execute(
            admin,
            WasmMsg::Migrate {
                contract_addr: cw721.to_string(),
                new_code_id: code_id_latest,
                msg: to_json_binary(&Cw721MigrateMsg::WithUpdate {
                    minter: Some(MINTER_ADDR.to_string()),
                    creator: Some(CREATOR_ADDR.to_string()),
                })
                .unwrap(),
            }
            .into(),
        )
        .unwrap();

        // legacy minter user cant mint
        let err: Cw721ContractError = app
            .execute_contract(
                legacy_creator_and_minter.clone(),
                cw721.clone(),
                &Cw721ExecuteMsg::<Empty, Empty, Empty>::Mint {
                    token_id: "1".to_string(),
                    owner: legacy_creator_and_minter.to_string(),
                    token_uri: Some("ipfs://new.uri".to_string()),
                    extension: Empty::default(),
                },
                &[],
            )
            .unwrap_err()
            .downcast()
            .unwrap();
        assert_eq!(err, Cw721ContractError::NotMinter {});

        // new minter can mint
        let minter = Addr::unchecked(MINTER_ADDR);
        mint_transfer_and_burn(&mut app, cw721.clone(), minter.clone(), "1".to_string());

        // check new mint query response works.
        #[allow(deprecated)]
        let m: MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(minter.to_string()));

        // check that the new response is backwards compatable when minter
        // is not None.
        #[allow(deprecated)]
        let m: v18::MinterResponse = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::Minter {},
            )
            .unwrap();
        assert_eq!(m.minter, Some(minter.to_string()));

        // check minter ownership query works
        let minter_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetMinterOwnership {},
            )
            .unwrap();
        assert_eq!(minter_ownership.owner, Some(minter));

        // check creator ownership query works
        let creator = Addr::unchecked(CREATOR_ADDR);
        let creator_ownership: Ownership<Addr> = app
            .wrap()
            .query_wasm_smart(
                &cw721,
                &Cw721QueryMsg::<
                    DefaultOptionalNftExtension,
                    DefaultOptionalCollectionExtension,
                    Empty,
                >::GetCreatorOwnership {},
            )
            .unwrap();
        assert_eq!(creator_ownership.owner, Some(creator));
    }
}

/// Test backward compatibility using instantiate msg from a 0.16 version on latest contract.
/// This ensures existing 3rd party contracts doesnt need to update as well.
#[test]
fn test_instantiate_016_msg() {
    use cw721_base_016 as v16;
    let mut app = App::default();
    let admin = || Addr::unchecked("admin");

    let code_id_latest = app.store_code(cw721_base_latest_contract());

    let cw721 = app
        .instantiate_contract(
            code_id_latest,
            admin(),
            &v16::InstantiateMsg {
                name: "collection".to_string(),
                symbol: "symbol".to_string(),
                minter: admin().into_string(),
            },
            &[],
            "cw721-base",
            Some(admin().into_string()),
        )
        .unwrap();

    // assert withdraw address is None
    let withdraw_addr: Option<String> = app
        .wrap()
        .query_wasm_smart(
            cw721,
            &Cw721QueryMsg::<
                DefaultOptionalNftExtension,
                DefaultOptionalCollectionExtension,
                Empty,
            >::GetWithdrawAddress {},
        )
        .unwrap();
    assert!(withdraw_addr.is_none());
}

#[test]
fn test_update_nft_metadata() {
    // --- setup ---
    let mut app = App::default();
    let admin = Addr::unchecked("admin");
    let code_id = app.store_code(cw721_base_latest_contract());
    let creator = Addr::unchecked(CREATOR_ADDR);
    let cw721 = app
        .instantiate_contract(
            code_id,
            creator.clone(),
            &Cw721InstantiateMsg::<DefaultOptionalCollectionExtension> {
                name: "collection".to_string(),
                symbol: "symbol".to_string(),
                minter: Some(MINTER_ADDR.to_string()),
                creator: None, // in case of none, sender is creator
                collection_info_extension: None,
                withdraw_address: None,
            },
            &[],
            "cw721-base",
            Some(admin.to_string()),
        )
        .unwrap();
    // mint
    let minter = Addr::unchecked(MINTER_ADDR);
    let nft_owner = Addr::unchecked(NFT_OWNER_ADDR);
    let nft_metadata_msg = NftExtensionMsg {
        image: Some("ipfs://foo.bar/image.png".to_string()),
        image_data: Some("image data".to_string()),
        external_url: Some("https://github.com".to_string()),
        description: Some("description".to_string()),
        name: Some("name".to_string()),
        attributes: Some(vec![Trait {
            trait_type: "trait_type".to_string(),
            value: "value".to_string(),
            display_type: Some("display_type".to_string()),
        }]),
        background_color: Some("background_color".to_string()),
        animation_url: Some("ssl://animation_url".to_string()),
        youtube_url: Some("file://youtube_url".to_string()),
    };
    app.execute_contract(
        minter,
        cw721.clone(),
        &Cw721ExecuteMsg::<
            DefaultOptionalNftExtensionMsg,
            DefaultOptionalCollectionExtensionMsg,
            Empty,
        >::Mint {
            token_id: "1".to_string(),
            owner: nft_owner.to_string(),
            token_uri: Some("ipfs://foo.bar/metadata.json".to_string()),
            extension: Some(nft_metadata_msg.clone()),
        },
        &[],
    )
    .unwrap();

    // check nft info
    let nft_info = query_nft_info(app.wrap(), &cw721, "1".to_string());
    assert_eq!(
        nft_info.token_uri,
        Some("ipfs://foo.bar/metadata.json".to_string())
    );
    assert_eq!(
        nft_info.extension,
        Some(NftExtension {
            image: Some("ipfs://foo.bar/image.png".to_string()),
            image_data: Some("image data".to_string()),
            external_url: Some("https://github.com".to_string()),
            description: Some("description".to_string()),
            name: Some("name".to_string()),
            attributes: Some(vec![Trait {
                trait_type: "trait_type".to_string(),
                value: "value".to_string(),
                display_type: Some("display_type".to_string()),
            }]),
            background_color: Some("background_color".to_string()),
            animation_url: Some("ssl://animation_url".to_string()),
            youtube_url: Some("file://youtube_url".to_string()),
        })
    );

    // nft owner cant update - only creator is allowed
    let err: Cw721ContractError = app
        .execute_contract(
            nft_owner,
            cw721.clone(),
            &Cw721ExecuteMsg::<
                DefaultOptionalNftExtensionMsg,
                DefaultOptionalCollectionExtensionMsg,
                Empty,
            >::UpdateNftInfo {
                token_id: "1".to_string(),
                token_uri: None,
                extension: Some(NftExtensionMsg {
                    name: Some("new name".to_string()),
                    description: Some("new description".to_string()),
                    image: None,
                    image_data: None,
                    external_url: None,
                    attributes: None,
                    background_color: None,
                    animation_url: None,
                    youtube_url: None,
                }),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, Cw721ContractError::NotCreator {});

    // update invalid token uri
    let err: Cw721ContractError = app
        .execute_contract(
            creator.clone(),
            cw721.clone(),
            &Cw721ExecuteMsg::<
                DefaultOptionalNftExtensionMsg,
                DefaultOptionalCollectionExtensionMsg,
                Empty,
            >::UpdateNftInfo {
                token_id: "1".to_string(),
                token_uri: Some("invalid".to_string()),
                extension: Some(NftExtensionMsg {
                    name: None,
                    description: None,
                    image: None,
                    image_data: None,
                    external_url: None,
                    attributes: None,
                    background_color: None,
                    animation_url: None,
                    youtube_url: None,
                }),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        Cw721ContractError::ParseError(ParseError::RelativeUrlWithoutBase)
    );

    // invalid image URL
    let err: Cw721ContractError = app
        .execute_contract(
            creator.clone(),
            cw721.clone(),
            &Cw721ExecuteMsg::<
                DefaultOptionalNftExtensionMsg,
                DefaultOptionalCollectionExtensionMsg,
                Empty,
            >::UpdateNftInfo {
                token_id: "1".to_string(),
                token_uri: None,
                extension: Some(NftExtensionMsg {
                    name: None,
                    description: None,
                    image: Some("invalid".to_string()),
                    image_data: None,
                    external_url: None,
                    attributes: None,
                    background_color: None,
                    animation_url: None,
                    youtube_url: None,
                }),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        Cw721ContractError::ParseError(ParseError::RelativeUrlWithoutBase)
    );

    // invalid external url
    let err: Cw721ContractError = app
        .execute_contract(
            creator.clone(),
            cw721.clone(),
            &Cw721ExecuteMsg::<
                DefaultOptionalNftExtensionMsg,
                DefaultOptionalCollectionExtensionMsg,
                Empty,
            >::UpdateNftInfo {
                token_id: "1".to_string(),
                token_uri: None,
                extension: Some(NftExtensionMsg {
                    name: None,
                    description: None,
                    image: None,
                    image_data: None,
                    external_url: Some("invalid".to_string()),
                    attributes: None,
                    background_color: None,
                    animation_url: None,
                    youtube_url: None,
                }),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        Cw721ContractError::ParseError(ParseError::RelativeUrlWithoutBase)
    );

    // invalid animation url
    let err: Cw721ContractError = app
        .execute_contract(
            creator.clone(),
            cw721.clone(),
            &Cw721ExecuteMsg::<
                DefaultOptionalNftExtensionMsg,
                DefaultOptionalCollectionExtensionMsg,
                Empty,
            >::UpdateNftInfo {
                token_id: "1".to_string(),
                token_uri: None,
                extension: Some(NftExtensionMsg {
                    name: None,
                    description: None,
                    image: None,
                    image_data: None,
                    external_url: None,
                    attributes: None,
                    background_color: None,
                    animation_url: Some("invalid".to_string()),
                    youtube_url: None,
                }),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        Cw721ContractError::ParseError(ParseError::RelativeUrlWithoutBase)
    );

    // invalid youtube url
    let err: Cw721ContractError = app
        .execute_contract(
            creator.clone(),
            cw721.clone(),
            &Cw721ExecuteMsg::<
                DefaultOptionalNftExtensionMsg,
                DefaultOptionalCollectionExtensionMsg,
                Empty,
            >::UpdateNftInfo {
                token_id: "1".to_string(),
                token_uri: None,
                extension: Some(NftExtensionMsg {
                    name: None,
                    description: None,
                    image: None,
                    image_data: None,
                    external_url: None,
                    attributes: None,
                    background_color: None,
                    animation_url: None,
                    youtube_url: Some("invalid".to_string()),
                }),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        Cw721ContractError::ParseError(ParseError::RelativeUrlWithoutBase)
    );

    // no image data (empty)
    app.execute_contract(
        creator.clone(),
        cw721.clone(),
        &Cw721ExecuteMsg::<
            DefaultOptionalNftExtensionMsg,
            DefaultOptionalCollectionExtensionMsg,
            Empty,
        >::UpdateNftInfo {
            token_id: "1".to_string(),
            token_uri: None,
            extension: Some(NftExtensionMsg {
                name: None,
                description: None,
                image: None,
                image_data: Some("".to_string()),
                external_url: None,
                attributes: None,
                background_color: None,
                animation_url: None,
                youtube_url: None,
            }),
        },
        &[],
    )
    .unwrap();
    // check nft info
    let nft_info = query_nft_info(app.wrap(), &cw721, "1".to_string());
    assert_eq!(
        nft_info.token_uri,
        Some("ipfs://foo.bar/metadata.json".to_string())
    );
    assert_eq!(
        nft_info.extension,
        Some(NftExtension {
            image: Some("ipfs://foo.bar/image.png".to_string()),
            image_data: None,
            external_url: Some("https://github.com".to_string()),
            description: Some("description".to_string()),
            name: Some("name".to_string()),
            attributes: Some(vec![Trait {
                trait_type: "trait_type".to_string(),
                value: "value".to_string(),
                display_type: Some("display_type".to_string()),
            }]),
            background_color: Some("background_color".to_string()),
            animation_url: Some("ssl://animation_url".to_string()),
            youtube_url: Some("file://youtube_url".to_string()),
        })
    );

    // no description (empty)
    app.execute_contract(
        creator.clone(),
        cw721.clone(),
        &Cw721ExecuteMsg::<
            DefaultOptionalNftExtensionMsg,
            DefaultOptionalCollectionExtensionMsg,
            Empty,
        >::UpdateNftInfo {
            token_id: "1".to_string(),
            token_uri: None,
            extension: Some(NftExtensionMsg {
                name: None,
                description: Some("".to_string()),
                image: None,
                image_data: None,
                external_url: None,
                attributes: None,
                background_color: None,
                animation_url: None,
                youtube_url: None,
            }),
        },
        &[],
    )
    .unwrap();
    // check nft info
    let nft_info = query_nft_info(app.wrap(), &cw721, "1".to_string());
    assert_eq!(
        nft_info.token_uri,
        Some("ipfs://foo.bar/metadata.json".to_string())
    );
    assert_eq!(
        nft_info.extension,
        Some(NftExtension {
            image: Some("ipfs://foo.bar/image.png".to_string()),
            image_data: None,
            external_url: Some("https://github.com".to_string()),
            description: None,
            name: Some("name".to_string()),
            attributes: Some(vec![Trait {
                trait_type: "trait_type".to_string(),
                value: "value".to_string(),
                display_type: Some("display_type".to_string()),
            }]),
            background_color: Some("background_color".to_string()),
            animation_url: Some("ssl://animation_url".to_string()),
            youtube_url: Some("file://youtube_url".to_string()),
        })
    );

    // no metadata name (empty)
    app.execute_contract(
        creator.clone(),
        cw721.clone(),
        &Cw721ExecuteMsg::<
            DefaultOptionalNftExtensionMsg,
            DefaultOptionalCollectionExtensionMsg,
            Empty,
        >::UpdateNftInfo {
            token_id: "1".to_string(),
            token_uri: None,
            extension: Some(NftExtensionMsg {
                name: Some("".to_string()),
                description: None,
                image: None,
                image_data: None,
                external_url: None,
                attributes: None,
                background_color: None,
                animation_url: None,
                youtube_url: None,
            }),
        },
        &[],
    )
    .unwrap();
    // check nft info
    let nft_info = query_nft_info(app.wrap(), &cw721, "1".to_string());
    assert_eq!(
        nft_info.token_uri,
        Some("ipfs://foo.bar/metadata.json".to_string())
    );
    assert_eq!(
        nft_info.extension,
        Some(NftExtension {
            image: Some("ipfs://foo.bar/image.png".to_string()),
            image_data: None,
            external_url: Some("https://github.com".to_string()),
            description: None,
            name: None,
            attributes: Some(vec![Trait {
                trait_type: "trait_type".to_string(),
                value: "value".to_string(),
                display_type: Some("display_type".to_string()),
            }]),
            background_color: Some("background_color".to_string()),
            animation_url: Some("ssl://animation_url".to_string()),
            youtube_url: Some("file://youtube_url".to_string()),
        })
    );

    // no background color (empty)
    app.execute_contract(
        creator.clone(),
        cw721.clone(),
        &Cw721ExecuteMsg::<
            DefaultOptionalNftExtensionMsg,
            DefaultOptionalCollectionExtensionMsg,
            Empty,
        >::UpdateNftInfo {
            token_id: "1".to_string(),
            token_uri: None,
            extension: Some(NftExtensionMsg {
                name: None,
                description: None,
                image: None,
                image_data: None,
                external_url: None,
                attributes: None,
                background_color: Some("".to_string()),
                animation_url: None,
                youtube_url: None,
            }),
        },
        &[],
    )
    .unwrap();
    // check nft info
    let nft_info = query_nft_info(app.wrap(), &cw721, "1".to_string());
    assert_eq!(
        nft_info.token_uri,
        Some("ipfs://foo.bar/metadata.json".to_string())
    );
    assert_eq!(
        nft_info.extension,
        Some(NftExtension {
            image: Some("ipfs://foo.bar/image.png".to_string()),
            image_data: None,
            external_url: Some("https://github.com".to_string()),
            description: None,
            name: None,
            attributes: Some(vec![Trait {
                trait_type: "trait_type".to_string(),
                value: "value".to_string(),
                display_type: Some("display_type".to_string()),
            }]),
            background_color: None,
            animation_url: Some("ssl://animation_url".to_string()),
            youtube_url: Some("file://youtube_url".to_string()),
        })
    );

    // invalid trait type (empty)
    let err: Cw721ContractError = app
        .execute_contract(
            creator.clone(),
            cw721.clone(),
            &Cw721ExecuteMsg::<
                DefaultOptionalNftExtensionMsg,
                DefaultOptionalCollectionExtensionMsg,
                Empty,
            >::UpdateNftInfo {
                token_id: "1".to_string(),
                token_uri: None,
                extension: Some(NftExtensionMsg {
                    name: None,
                    description: None,
                    image: None,
                    image_data: None,
                    external_url: None,
                    attributes: Some(vec![Trait {
                        trait_type: "".to_string(),
                        value: "value".to_string(),
                        display_type: Some("display_type".to_string()),
                    }]),
                    background_color: None,
                    animation_url: None,
                    youtube_url: None,
                }),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, Cw721ContractError::TraitTypeEmpty {});

    // invalid trait value (empty)
    let err: Cw721ContractError = app
        .execute_contract(
            creator.clone(),
            cw721.clone(),
            &Cw721ExecuteMsg::<
                DefaultOptionalNftExtensionMsg,
                DefaultOptionalCollectionExtensionMsg,
                Empty,
            >::UpdateNftInfo {
                token_id: "1".to_string(),
                token_uri: None,
                extension: Some(NftExtensionMsg {
                    name: None,
                    description: None,
                    image: None,
                    image_data: None,
                    external_url: None,
                    attributes: Some(vec![Trait {
                        trait_type: "trait_type".to_string(),
                        value: "".to_string(),
                        display_type: Some("display_type".to_string()),
                    }]),
                    background_color: None,
                    animation_url: None,
                    youtube_url: None,
                }),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, Cw721ContractError::TraitValueEmpty {});

    // invalid trait display type (empty)
    let err: Cw721ContractError = app
        .execute_contract(
            creator.clone(),
            cw721.clone(),
            &Cw721ExecuteMsg::<
                DefaultOptionalNftExtensionMsg,
                DefaultOptionalCollectionExtensionMsg,
                Empty,
            >::UpdateNftInfo {
                token_id: "1".to_string(),
                token_uri: None,
                extension: Some(NftExtensionMsg {
                    name: None,
                    description: None,
                    image: None,
                    image_data: None,
                    external_url: None,
                    attributes: Some(vec![Trait {
                        trait_type: "trait_type".to_string(),
                        value: "value".to_string(),
                        display_type: Some("".to_string()),
                    }]),
                    background_color: None,
                    animation_url: None,
                    youtube_url: None,
                }),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, Cw721ContractError::TraitDisplayTypeEmpty {});

    // proper update
    let new_nft_metadata_msg = NftExtensionMsg {
        image: None, // set to none to ensure it is unchanged
        image_data: Some("image data2".to_string()),
        external_url: Some("https://github.com2".to_string()),
        description: Some("description2".to_string()),
        name: Some("name2".to_string()),
        attributes: Some(vec![Trait {
            trait_type: "trait_type2".to_string(),
            value: "value2".to_string(),
            display_type: Some("display_type2".to_string()),
        }]),
        background_color: Some("background_color2".to_string()),
        animation_url: Some("ssl://animation_url2".to_string()),
        youtube_url: Some("file://youtube_url2".to_string()),
    };
    app.execute_contract(
        creator,
        cw721.clone(),
        &Cw721ExecuteMsg::<
            DefaultOptionalNftExtensionMsg,
            DefaultOptionalCollectionExtensionMsg,
            Empty,
        >::UpdateNftInfo {
            token_id: "1".to_string(),
            token_uri: Some("ipfs://foo.bar/metadata2.json".to_string()),
            extension: Some(new_nft_metadata_msg.clone()),
        },
        &[],
    )
    .unwrap();
    // check token uri and extension
    let nft_info = query_nft_info(app.wrap(), &cw721, "1".to_string());
    assert_eq!(
        nft_info.token_uri,
        Some("ipfs://foo.bar/metadata2.json".to_string())
    );
    assert_eq!(
        nft_info.extension,
        Some(NftExtension {
            image: Some("ipfs://foo.bar/image.png".to_string()), // this is unchanged
            image_data: Some("image data2".to_string()),
            external_url: Some("https://github.com2".to_string()),
            description: Some("description2".to_string()),
            name: Some("name2".to_string()),
            attributes: Some(vec![Trait {
                trait_type: "trait_type2".to_string(),
                value: "value2".to_string(),
                display_type: Some("display_type2".to_string()),
            }]),
            background_color: Some("background_color2".to_string()),
            animation_url: Some("ssl://animation_url2".to_string()),
            youtube_url: Some("file://youtube_url2".to_string()),
        })
    );
    // check num tokens
    let num_tokens: NumTokensResponse = app
        .wrap()
        .query_wasm_smart(
            &cw721,
            &Cw721QueryMsg::<
                DefaultOptionalNftExtension,
                DefaultOptionalCollectionExtension,
                Empty,
            >::NumTokens {},
        )
        .unwrap();
    assert_eq!(num_tokens.count, 1);
}
