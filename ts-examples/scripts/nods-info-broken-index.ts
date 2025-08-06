import {getContractAddresses, initClient} from "../lib/clientService";
import {TributeQueryClient} from "../clients/tribute/Tribute.client";
import {RUN_DATE, RUN_DATE_TS} from "../config";
import {NodQueryClient} from "../clients/nod/Nod.client";
import {TokensResponse} from "../clients/nod/Nod.types";

async function main() {
    const {walletClient} = await initClient()

    let height = await walletClient.getHeight()
    console.log("Current Height:", height)

    const nodeAddress = await getContractAddresses('TRIBUTE_CONTRACT_ADDRESS');
    console.log("Node address: ", nodeAddress)
    const nodClient = new NodQueryClient(walletClient, nodeAddress)
    let tokensResp = await nodClient.numTokens();
    console.log("Number of Nod tokens: ", tokensResp)

    let last_nod_id: string | undefined;
    let done = false;
    while (!done) {
        let response : TokensResponse = await nodClient.allTokens({
            queryOrder: "descending",
            startAfter: last_nod_id,
        })

        for (let tokenId of response.tokens) {
            let tokenInfo = await nodClient.nftInfo({tokenId: tokenId})
            try {
                let _token_by_owner = await nodClient.tokens({limit: 1, owner: tokenInfo.owner})
                console.log("Ok index for token : ", tokenId, ", owner: ", tokenInfo.owner)
            } catch (e) {
                if (e instanceof Error && e.message.includes("pk not found")) {
                    console.log("Broken index for token : ", tokenId, ", owner: ", tokenInfo.owner)
                } else {
                    console.log("Error: ", e)
                }
            }
        }

        if (response.tokens.length == 0) {
            done = true;
        } else {
            last_nod_id = response.tokens[response.tokens.length - 1];
        }
    }
}

main();
