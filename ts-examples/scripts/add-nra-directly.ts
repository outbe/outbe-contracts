import {getContractAddresses, initClient} from "../lib/clientService";
import {AgentNraClient} from "../clients/agent-nra/AgentNra.client";
import {AgentExt, AgentInput} from "../clients/agent-nra/AgentNra.types";
import {NRA_AGENTS, TX_FEE} from "../config";

async function main() {
  console.log("Starting NRA agents addition...");

  const {walletClient, account} = await initClient();
  console.log("Using address:", account.address);

  const agentNraAddress = await getContractAddresses('AGENT_NRA_CONTRACT_ADDRESS');
  const agentNraClient = new AgentNraClient(walletClient, account.address, agentNraAddress);

  for (let i = 0; i < NRA_AGENTS.length; i++) {
    const agent = NRA_AGENTS[i];

    try {
      console.log(`\nAdding agent ${i + 1}: ${agent.name}`);

      const agentInput: AgentInput = {
        name: agent.name,
        email: agent.email,
        jurisdictions: [],
        endpoint: null,
        metadata_json: null,
        docs_uri: [],
        discord: null,
        avg_cu: null,
        ext: {nra: {}} as AgentExt
      };

      const result = await agentNraClient.owner(
        {
          add_nra_directly: {
            address: agent.address,
            agent: agentInput
          }
        },
        TX_FEE
      );

      console.log(`✅ Success! TX: ${result.transactionHash}`);

    } catch (error) {
      console.log(`❌ Failed: ${error}`);
    }
  }

  console.log("\nDone!");
}

main().catch(console.error);
