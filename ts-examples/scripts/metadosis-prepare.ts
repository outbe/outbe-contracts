import {RUN_DATE, TX_FEE} from "../config";
import {getContractAddresses, initClient} from "../lib/clientService";
import {MetadosisClient} from "../clients/metadosis/Metadosis.client";
import {MetadosisInfoResponse} from "../clients/metadosis/Metadosis.types";


async function main() {
  const {walletClient, account} = await initClient()

  let balance = await walletClient.getBalance(account.address, "unit")
  console.log("Balance: ", balance)
  let height = await walletClient.getHeight()
  console.log("Current Height: ", height)

  const metadosisContractAddress = await getContractAddresses('METADOSIS_CONTRACT_ADDRESS')
  const metadosisClient = new MetadosisClient(walletClient, account.address, metadosisContractAddress)

  let tx_prepare = await metadosisClient.prepare({runDate: RUN_DATE}, TX_FEE)
  console.log(`Prepare Metadosis, timestamp:${RUN_DATE} tx: ${tx_prepare.transactionHash}`)

  let info: MetadosisInfoResponse = await metadosisClient.metadosisInfo()
  console.log("info:")
  console.log(JSON.stringify(info, null, 2))
}

main().catch(console.error);
