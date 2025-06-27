import {CosmWasmClient} from "@cosmjs/cosmwasm-stargate";

const TRIBUTE_CONTRACT_ADDRESS = "outbe1rle2gp6qcz6jvzjrhz2vwkvew286e3pd3wn65l5mwltzw6u3psgsjdg3t7"

// Example of how to use the function:
const endpoint = "https://rpc.dev.outbe.net";

async function main() {
    let client = await CosmWasmClient.connect(endpoint);
    let height = await client.getHeight()
    console.log("Current Height: ", height)

    let tokensResp = await client.queryContractSmart(TRIBUTE_CONTRACT_ADDRESS, {
        num_tokens: {}
    })
    console.log("Number of Tribute tokens: ", tokensResp)


    let dailyTributes = await client.queryContractSmart(TRIBUTE_CONTRACT_ADDRESS, {
        daily_tributes: {date: "1750982400"}
    })
    console.log("distribution: ")
    console.log(JSON.stringify(dailyTributes, null, 2))
}


main();
