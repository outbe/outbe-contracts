import {initClient} from "../lib/clientService";
import {ContractRegistryQueryClient} from "../clients/contract-registry/ContractRegistry.client";
import {CONTRACT_REGISTRY_ADDRESS} from "../config";

async function main() {
  const {walletClient} = await initClient()

  let height = await walletClient.getHeight()
  console.log("Current Height:", height)

  const registryClient = new ContractRegistryQueryClient(walletClient, CONTRACT_REGISTRY_ADDRESS!)

  let last_id: string | undefined;
  let done = false;

  while (!done) {
    const response = await registryClient.allDeployments({
      limit: 10,
      startAfter: last_id,
    });
    console.log("Response", response)

    if (response.deployments.length == 0) {
      done = true;
    } else {
      last_id = response.deployments[response.deployments.length - 1].commit_id;
    }
  }
}

main().catch(console.error);
