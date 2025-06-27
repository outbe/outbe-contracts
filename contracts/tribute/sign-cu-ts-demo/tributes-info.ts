import {CosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {RPC_ENDPOINT, TRIBUTE_CONTRACT_ADDRESS} from "./consts";

async function main() {
    let client = await CosmWasmClient.connect(RPC_ENDPOINT);
    let height = await client.getHeight()
    console.log("Current Height: ", height)

    let tokensResp = await client.queryContractSmart(TRIBUTE_CONTRACT_ADDRESS, {
        num_tokens: {}
    })
    console.log("Number of Tribute tokens: ", tokensResp)


    let dailyTributes = await client.queryContractSmart(TRIBUTE_CONTRACT_ADDRESS, {
        daily_tributes: {date: 1750982400 }
    })
    console.log("distribution: ")
    console.log(JSON.stringify(dailyTributes, null, 2))
}


main();
