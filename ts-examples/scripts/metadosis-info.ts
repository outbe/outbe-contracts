import {getContractAddresses, initClient} from "../lib/clientService";
import {MetadosisQueryClient} from "../clients/metadosis/Metadosis.client";
import {MetadosisInfoResponse} from "../clients/metadosis/Metadosis.types";
import {NodQueryClient} from "../clients/nod/Nod.client";
import {NumTokensResponse} from "../clients/nod/Nod.types";

async function main() {
  const {walletClient} = await initClient()

  let height = await walletClient.getHeight()
  console.log("Current Height: ", height)

  const metadosisContractAddress = await getContractAddresses('METADOSIS_CONTRACT_ADDRESS')
  const metadosisClient = new MetadosisQueryClient(walletClient, metadosisContractAddress)
  let info: MetadosisInfoResponse = await metadosisClient.metadosisInfo()
  console.log("info: ")
  console.log(JSON.stringify(info, null, 2))


  const nodContractAddress = await getContractAddresses('NOD_CONTRACT_ADDRESS')
  const nodClient = new NodQueryClient(walletClient, nodContractAddress)
  let tokensResp: NumTokensResponse = await nodClient.numTokens()
  console.log("Number of Nod tokens: ", tokensResp)


}


main();
