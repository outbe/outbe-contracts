import {CosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {RPC_ENDPOINT, METADOSIS_CONTRACT_ADDRESS, NOD_CONTRACT_ADDRESS} from "./consts";

async function main() {
    let client = await CosmWasmClient.connect(RPC_ENDPOINT);
    let height = await client.getHeight()
    console.log("Current Height: ", height)

    let runInfo = await client.queryContractSmart(METADOSIS_CONTRACT_ADDRESS, {
        daily_runs: {}
    })
    console.log("runInfo: ")
    console.log(JSON.stringify(runInfo, null, 2))

    let tokensResp = await client.queryContractSmart(NOD_CONTRACT_ADDRESS, {
        num_tokens: {}
    })
    console.log("Number of Nod tokens: ", tokensResp)


    let distribution = await client.queryContractSmart(METADOSIS_CONTRACT_ADDRESS, {
        tributes_distribution: {}
    })
    console.log("distribution: ")
    console.log(JSON.stringify(distribution, null, 2))
}


main();
