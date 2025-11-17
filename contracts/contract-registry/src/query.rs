use crate::state::{DEPLOYMENTS, LATEST_DEPLOYMENT};
use crate::types::{AllDeploymentsResponse, Deployment, GetDeploymentResponse};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AllDeploymentsResponse)]
    AllDeployments {
        start_after: Option<String>, // commit_id
        limit: Option<u32>,
    },

    /// Returns the deployment by commit_id.
    /// If commit_id is not provided, returns the latest deployment
    #[returns(GetDeploymentResponse)]
    GetDeployment { commit_id: Option<String> },

    #[returns(cosmwasm_std::Binary)]
    Ownable(),
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AllDeployments { start_after, limit } => {
            to_json_binary(&query_all_deployments(deps, start_after, limit)?)
        }
        QueryMsg::GetDeployment { commit_id } => {
            to_json_binary(&query_get_deployment(deps, commit_id)?)
        }
        QueryMsg::Ownable() => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
    }
}

fn query_get_deployment(deps: Deps, commit_id: Option<String>) -> StdResult<GetDeploymentResponse> {
    let latest_deployment_commit = LATEST_DEPLOYMENT
        .load(deps.storage)
        .unwrap_or("none".to_string());
    let commit = commit_id.unwrap_or(latest_deployment_commit.clone());
    let deployment = DEPLOYMENTS.load(deps.storage, commit)?;
    Ok(GetDeploymentResponse {
        deployment: Deployment {
            commit_id: deployment.commit_id.clone(),
            contracts: deployment.contracts,
            is_latest: deployment.commit_id == latest_deployment_commit,
        },
    })
}

pub const DEFAULT_LIMIT: u32 = 10;
pub const MAX_LIMIT: u32 = 1000;

// Query
fn query_all_deployments(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllDeploymentsResponse> {
    let start = start_after.as_deref().map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).max(MAX_LIMIT) as usize;

    let latest_deployment_commit = LATEST_DEPLOYMENT
        .load(deps.storage)
        .unwrap_or("none".to_string());

    let res: Vec<Deployment> = DEPLOYMENTS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, deployment) = item?;
            Ok(Deployment {
                commit_id: deployment.commit_id.clone(),
                contracts: deployment.contracts.clone(),
                is_latest: deployment.commit_id == latest_deployment_commit,
            })
        })
        .collect::<StdResult<Vec<Deployment>>>()?;

    Ok(AllDeploymentsResponse { deployments: res })
}
