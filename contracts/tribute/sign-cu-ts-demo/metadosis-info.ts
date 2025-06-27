import {CosmWasmClient} from "@cosmjs/cosmwasm-stargate";


const METADOSIS_CONTRACT_ADDRESS = "outbe12j3gvpmcte38khlkez28wysp3dwrw0gwwss7w5qxg7hzxmk9ku9s463evk"
const NOD_CONTRACT_ADDRESS = "outbe12mg78yvda3hddecgyqvn6znase6ad324pf0xzagjkek0ym9nvy6qd6trg6"


// Example of how to use the function:
const endpoint = "https://rpc.dev.outbe.net";

async function main() {
    let client = await CosmWasmClient.connect(endpoint);
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
