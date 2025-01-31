use crate::msg::AppMigrateMsg;
use crate::replies::{TASK_CREATE_REPLY_ID, TASK_REMOVE_REPLY_ID};
use crate::{
    error::AppError,
    handlers,
    msg::{AppExecuteMsg, AppInstantiateMsg, AppQueryMsg},
    replies::{self, INSTANTIATE_REPLY_ID},
};
use abstract_app::AppContract;
use cosmwasm_std::Response;

/// The version of your app
pub const CRONCAT_MODULE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const CRONCAT_ID: &str = "croncat:app";

/// The type of the result returned by your app's entry points.
pub type CroncatResult<T = Response> = Result<T, AppError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type CroncatApp =
    AppContract<AppError, AppInstantiateMsg, AppExecuteMsg, AppQueryMsg, AppMigrateMsg>;

const CRONCAT_APP: CroncatApp = CroncatApp::new(CRONCAT_ID, CRONCAT_MODULE_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler)
    .with_replies(&[
        (INSTANTIATE_REPLY_ID, replies::instantiate_reply),
        (TASK_CREATE_REPLY_ID, replies::create_task_reply),
        (TASK_REMOVE_REPLY_ID, replies::task_remove_reply),
    ]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(CRONCAT_APP, CroncatApp);

// Small helper
pub(crate) fn check_users_balance_nonempty(
    deps: cosmwasm_std::Deps,
    proxy_addr: cosmwasm_std::Addr,
    manager_addr: cosmwasm_std::Addr,
) -> Result<bool, AppError> {
    let coins: Vec<cw20::Cw20CoinVerified> = deps.querier.query_wasm_smart(
        manager_addr,
        &croncat_sdk_manager::msg::ManagerQueryMsg::UsersBalances {
            address: proxy_addr.into_string(),
            from_index: None,
            // One is enough to know
            limit: Some(1),
        },
    )?;
    Ok(!coins.is_empty())
}

pub(crate) fn get_croncat_contract(
    querier: &cosmwasm_std::QuerierWrapper,
    croncat_factory_address: cosmwasm_std::Addr,
    croncat_contract_name: &str,
    croncat_version: &str,
) -> cosmwasm_std::StdResult<cosmwasm_std::Addr> {
    // Parse string to a version.
    let croncat_version_parsed = croncat_version
        .split('.')
        .map(|ver| ver.parse().unwrap())
        .collect::<Vec<u8>>();

    // Raw query the factory
    let contract_addr = croncat_factory::state::CONTRACT_ADDRS
        .query(
            querier,
            croncat_factory_address,
            (croncat_contract_name, &croncat_version_parsed),
        )?
        .unwrap();

    Ok(contract_addr)
}
