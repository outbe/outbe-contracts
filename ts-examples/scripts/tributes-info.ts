import {getContractAddresses, initClient} from "../lib/clientService";
import {TributeQueryClient} from "../clients/tribute/Tribute.client";

async function main() {
  const {walletClient} = await initClient()

  let height = await walletClient.getHeight()
  console.log("Current Height: ", height)

  const tributeContractAddress = await getContractAddresses('TRIBUTE_CONTRACT_ADDRESS');
  const tributeClient = new TributeQueryClient(walletClient, tributeContractAddress)
  let tokensResp = await tributeClient.numTokens();
  console.log("Number of Tribute tokens: ", tokensResp)

  // let dailyTributes = await client.queryContractSmart(TRIBUTE_CONTRACT_ADDRESS, {
  //     daily_tributes: {date: 1750982400 }
  // })
  // console.log("distribution: ")
  // console.log(JSON.stringify(dailyTributes, null, 2))
}


main();
