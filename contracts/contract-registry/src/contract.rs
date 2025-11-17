use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{DeploymentInfoState, DEPLOYMENTS, LATEST_DEPLOYMENT};
use crate::types::Deployment;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "outbe.net:contract-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Publish { deployment } => {
            cw_ownable::assert_owner(deps.storage, &info.sender)?;
            execute_publish_deployment(deps, deployment)
        }
        ExecuteMsg::Ownable(action) => {
            let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(Response::new()
                .add_attribute("update_ownership", info.sender.to_string())
                .add_attributes(ownership.into_attributes()))
        }
    }
}

fn execute_publish_deployment(
    deps: DepsMut,
    deployment: Deployment,
) -> Result<Response, ContractError> {
    let commit = deployment.commit_id.clone();
    DEPLOYMENTS.save(
        deps.storage,
        commit.clone(),
        &DeploymentInfoState {
            commit_id: deployment.commit_id.clone(),
            contracts: deployment.contracts.clone(),
        },
    )?;
    if deployment.is_latest {
        LATEST_DEPLOYMENT.save(deps.storage, &commit)?;
    }

    Ok(Response::new()
        .add_attribute("action", "contract-registry::deployment")
        .add_event(
            Event::new("contract-registry::deployment")
                .add_attribute("commit_id", deployment.commit_id)
                .add_attribute("is_latest", deployment.is_latest.to_string())
                .add_attributes(
                    deployment
                        .contracts
                        .into_iter()
                        .map(|c| (c.name, c.address.to_string())),
                ),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::query;
    use crate::query::{QueryMsg, QueryMsg::GetDeployment};
    use crate::types::{AllDeploymentsResponse, ContractInfo, GetDeploymentResponse};
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

    #[test]
    fn test_all() {
        let mut app = App::default();
        let owner_addr = app.api().addr_make("owner");

        // Deploy contract
        let contract_code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(contract_code));
        let contract_addr = app
            .instantiate_contract(
                code_id,
                owner_addr.clone(),
                &InstantiateMsg {},
                &[],
                "contract-registry",
                None,
            )
            .unwrap();

        // Execute update contracts
        let new_contract = ContractInfo {
            name: "name2".to_string(),
            address: Addr::unchecked("addr2"),
        };
        let msg = ExecuteMsg::Publish {
            deployment: Deployment {
                commit_id: "commit_id2".to_string(),
                contracts: vec![new_contract.clone()],
                is_latest: true,
            },
        };
        app.execute_contract(owner_addr.clone(), contract_addr.clone(), &msg, &[])
            .unwrap();

        // Query contract
        let query_msg = GetDeployment { commit_id: None };
        let query_response: GetDeploymentResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &query_msg)
            .unwrap();
        assert_eq!(
            query_response
                .deployment
                .contracts
                .iter()
                .find(|c| c.name == new_contract.name)
                .unwrap()
                .address,
            new_contract.address
        );

        // Query list contracts
        let query_msg = QueryMsg::AllDeployments {
            start_after: None,
            limit: None,
        };
        let query_response: AllDeploymentsResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &query_msg)
            .unwrap();
        assert_eq!(query_response.deployments.len(), 1);
    }
}
