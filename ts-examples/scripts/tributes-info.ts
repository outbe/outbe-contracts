import {getContractAddresses, initClient} from "../lib/clientService";
import {TributeQueryClient} from "../clients/tribute/Tribute.client";
import {RUN_DATE, RUN_DATE_TS} from "../config";

async function main() {
    const {walletClient} = await initClient()

    let height = await walletClient.getHeight()
    console.log("Current Height:", height)
    console.log("Using RUN_DATE", RUN_DATE, ":", RUN_DATE_TS())

    const tributeContractAddress = await getContractAddresses('TRIBUTE_CONTRACT_ADDRESS');
    const tributeClient = new TributeQueryClient(walletClient, tributeContractAddress)
    let tokensResp = await tributeClient.numTokens();
    console.log("Number of Tribute tokens: ", tokensResp)

    let daily_tokens = await tributeClient.dailyTributes({
        date: RUN_DATE_TS(),
    })
    console.log("daily tokens:")
    console.log(JSON.stringify(daily_tokens, null, 2))
}

main();
