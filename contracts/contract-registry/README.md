# Contract Registry

The contract registry is a contract that keeps track of all the
contracts that are deployed on the network. It is a simple contract that
allows for the registration of contracts and the retrieval of contracts
by name.

We are using a mapping to store the contracts with the name as the key.
This allows for quick retrieval of the contract by name.

## Specifications

Here are the messages for the Contract Registry smart contract:

``` rust
#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Publish { deployment: Deployment },
    Ownable(ownable::msg::ExecuteMsg),
}

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
    Ownable(ownable::msg::QueryMsg),
}
```

You can find more about the messages in the [msg.rs](./src/msg.rs) file.
Please have a look at to the root [README.md](../README.md) for how to
use these messages.

## Query examples

```shell
gemchaind $NODE query wasm contract-state smart $CONTRACT_REGISTRY_CONTRACT_ADDRESS '{"all_deployments": {}}'
```
