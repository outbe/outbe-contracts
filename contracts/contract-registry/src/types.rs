use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

/// Helper type for identifying commit id
pub type CommitId = String;

#[cw_serde]
pub struct DeploymentInfo {
    pub commit_id: CommitId,
    pub contracts: Vec<ContractInfo>,
}

#[cw_serde]
pub struct ContractInfo {
    /// Contract name, should be the same as it's env variable on CI
    pub name: String,
    /// Contract address
    pub address: Addr,
}

#[cw_serde]
pub struct Deployment {
    pub commit_id: CommitId,
    pub contracts: Vec<ContractInfo>,
    /// identifies whenever to mark this deployment as "latest"
    pub is_latest: bool,
}

#[cw_serde]
pub struct AllDeploymentsResponse {
    pub deployments: Vec<Deployment>,
}

#[cw_serde]
pub struct GetDeploymentResponse {
    pub deployment: Deployment,
}
