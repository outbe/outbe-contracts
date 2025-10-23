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

  let response: TokensResponse = await nodClient.allTokens({
    queryOrder: "descending",
    startAfter: last_nod_id,
  })
  console.log("Fetching one page of Nod tokens:")
  for (let token_id of response.tokens) {
    await nodClient.nftInfo({tokenId: token_id}).then(info => {
      console.log("Nod data: ", info)
    })
  }
  // Uncomment if need to query all tokens
  // while (!done) {
  //   let response: TokensResponse = await nodClient.allTokens({
  //     queryOrder: "descending",
  //     startAfter: last_nod_id,
  //   })
  //   console.log("Nod ids batch size =", response.tokens.length, ", data: ", response.tokens.join(", "))
  //   if (response.tokens.length == 0) {
  //     done = true;
  //   } else {
  //     last_nod_id = response.tokens[response.tokens.length - 1];
  //   }
  // }
}

main();
