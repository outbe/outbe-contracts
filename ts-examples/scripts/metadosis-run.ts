import {NUMBER_OF_METADOSIS_RUNS, RUN_DATE, TX_FEE} from "../config";
import {getContractAddresses, initClient} from "../lib/clientService";
import {MetadosisClient} from "../clients/metadosis/Metadosis.client";
import {HistoryResponse, MetadosisInfoResponse} from "../clients/metadosis/Metadosis.types";
import {NodQueryClient} from "../clients/nod/Nod.client";

async function main() {
  const {walletClient, account} = await initClient()

  let balance = await walletClient.getBalance(account.address, "unit")
  console.log("Balance: ", balance)
  let height = await walletClient.getHeight()
  console.log("Current Height: ", height)

  const metadosisContractAddress = await getContractAddresses('METADOSIS_CONTRACT_ADDRESS')
  const metadosisClient = new MetadosisClient(walletClient, account.address, metadosisContractAddress)

  const nodClient = new NodQueryClient(walletClient, await getContractAddresses('NOD_CONTRACT_ADDRESS'))

  for (let i = 1; i <= NUMBER_OF_METADOSIS_RUNS; i++) {
    let tx = await metadosisClient.execute({runDate: RUN_DATE}, TX_FEE)
    console.log(i + ": Executed Metadosis, tx ", tx.transactionHash)
    console.log("Number of Nod tokens (after run): ", await nodClient.numTokens())
  }

  let info: MetadosisInfoResponse = await metadosisClient.metadosisInfo()
  console.log("info:")
  console.log(JSON.stringify(info, null, 2))
  console.log("")
  console.log("")
  let history: HistoryResponse = await metadosisClient.history()
  console.log("history:")
  console.log(JSON.stringify(history, null, 2))
}

main().catch(console.error);
