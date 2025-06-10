use crate::types::{ContractInfo, DeploymentInfo};
use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct DeploymentInfoState {
    pub commit_id: String,
    pub contracts: Vec<ContractInfo>,
}

impl From<DeploymentInfo> for DeploymentInfoState {
    fn from(value: DeploymentInfo) -> DeploymentInfoState {
        DeploymentInfoState {
            commit_id: value.commit_id,
            contracts: value.contracts,
        }
    }
}

/// commit id -> Deployment
pub const DEPLOYMENTS: Map<&str, DeploymentInfoState> = Map::new("deployments");

pub const LATEST_DEPLOYMENT: Item<String> = Item::new("latest_deployment");
