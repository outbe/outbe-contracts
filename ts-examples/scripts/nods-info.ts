import {getContractAddresses, initClient} from "../lib/clientService";
import {NodQueryClient} from "../clients/nod/Nod.client";
import {TokensResponse} from "../clients/nod/Nod.types";

async function main() {
  const {walletClient} = await initClient()

  let height = await walletClient.getHeight()
  console.log("Current Height:", height)

  const nodeAddress = await getContractAddresses('NOD_CONTRACT_ADDRESS');
  console.log("Node address: ", nodeAddress)
  const nodClient = new NodQueryClient(walletClient, nodeAddress)
  let tokensResp = await nodClient.numTokens();
  console.log("Number of Nod tokens: ", tokensResp)

  let last_nod_id: string | undefined;
  let done = false;

  while (!done) {
    let response: TokensResponse = await nodClient.allTokens({
      queryOrder: "descending",
      startAfter: last_nod_id,
    })
    console.log("Nod ids batch size =", response.tokens.length)
    for (let token_id of response.tokens) {
      await nodClient.nftInfo({tokenId: token_id}).then(info => {
        if (info.owner == "outbe10p4p27fccqm2hrxqvzhcny7xvatg63576pvxc8")
            console.log("Nod data: ", info)
      })
    }

    if (response.tokens.length == 0) {
      done = true;
    } else {
      last_nod_id = response.tokens[response.tokens.length - 1];
    }
  }
}

main().catch(console.error);
