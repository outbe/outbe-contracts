import { getContractAddresses, initClient } from "../lib/clientService";
import { AgentNraClient } from "../clients/agent-nra/AgentNra.client";
import { AgentInput, AgentExt } from "../clients/agent-nra/AgentNra.types";
import { TX_FEE, NRA_AGENTS } from "../config";

async function addNraAgent(
    agentNraClient: AgentNraClient,
    agentData: { address: string; name: string; email: string },
    index: number
) {
    // Prepare agent input data with defaults
    const agentInput: AgentInput = {
        name: agentData.name,
        email: agentData.email,
        jurisdictions: [],
        endpoint: null,
        metadata_json: null,
        docs_uri: [],
        discord: null,
        avg_cu: null,
        ext: { nra: {} } as AgentExt // NRA agent extension
    };

    try {
        console.log(`Adding NRA agent ${index + 1}...`);
        console.log("Agent data:", {
            address: agentData.address,
            name: agentInput.name,
            email: agentInput.email,
        });

        // Execute add_nra_directly
        const result = await agentNraClient.owner(
            {
                add_nra_directly: {
                    address: agentData.address,
                    agent: agentInput
                }
            },
            TX_FEE
        );
        return { success: true, txHash: result.transactionHash };

    } catch (error) {
        console.error(`Error adding NRA agent ${index + 1}:`, error);

        // Check specific error types
        if (error instanceof Error) {
            if (error.message.includes("Unauthorized")) {
                console.error("💡 Make sure you are the contract owner");
            } else if (error.message.includes("AgentAlreadyExists")) {
                console.error("💡 Agent with this address already exists");
            }
        }

        return { success: false, error: error };
    }
}

async function main() {
    console.log("Starting NRA agents direct addition...");

    const { walletClient, account } = await initClient();

    let height = await walletClient.getHeight();
    console.log("Current Height: ", height);
    console.log("Using runner address: ", account.address);

    // Get agent-nra contract address
    const agentNraAddress = await getContractAddresses('agent-nra');
    const agentNraClient = new AgentNraClient(walletClient, account.address, agentNraAddress);

    let successCount = 0;
    let failureCount = 0;
    const results = [];

    // Process each agent
    for (let i = 0; i < NRA_AGENTS.length; i++) {
        const agentData = NRA_AGENTS[i];
        console.log(`\n--- Processing agent ${i + 1}/${NRA_AGENTS.length} ---`);

        const result = await addNraAgent(agentNraClient, agentData, i);
        results.push({ agent: agentData, result });

        if (result.success) {
            successCount++;
        } else {
            failureCount++;
        }

    }



    if (successCount > 0) {
        console.log(`✅ Successfully added: ${successCount} agents`);
        console.log("\n Successful transactions:");
        results.forEach((r, i) => {
            if (r.result.success) {
                console.log(`${i + 1}. ${r.agent.name} - ${r.result.txHash}`);
            }
        });
    }

    if (failureCount > 0) {
        console.log(`❌ Failed to add: ${failureCount} agents`);
        console.log("\n Failed transactions:");
        results.forEach((r, i) => {
            if (!r.result.success) {
                console.log(`${i + 1}. ${r.agent.name} - ${r.result.error?.message || 'Unknown error'}`);
            }
        });
    }
}

main().catch(console.error);