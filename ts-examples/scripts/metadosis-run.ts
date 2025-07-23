import {NUMBER_OF_METADOSIS_RUNS, TX_FEE} from "../config";
import {getContractAddresses, initClient} from "../lib/clientService";
import {getCurrentUnixTimestamp, normalize_to_date} from "../lib/utils";
import {MetadosisClient} from "../clients/metadosis/Metadosis.client";


async function main() {
  const {walletClient, account} = await initClient()

  let balance = await walletClient.getBalance(account.address, "unit")
  console.log("Balance: ", balance)
  let height = await walletClient.getHeight()
  console.log("Current Height: ", height)


  let current_timestamp = getCurrentUnixTimestamp();
  let current_date = normalize_to_date(current_timestamp);
  console.log("Current timestamp: ", current_timestamp)
  console.log("Current date: ", current_date)

  const metadosisContractAddress = await getContractAddresses('METADOSIS_CONTRACT_ADDRESS')
  const metadosisClient = new MetadosisClient(walletClient, account.address, metadosisContractAddress)
  const runDate = '1751288793'

  for (let i = 1; i <= NUMBER_OF_METADOSIS_RUNS; i++) {
    let tx = await metadosisClient.execute({runDate}, TX_FEE)
    console.log(i + ": Executed Metadosis, tx ", tx.transactionHash)
  }

  let runInfo = await metadosisClient.dailyRuns()
  console.log("runInfo:")
  console.log(JSON.stringify(runInfo, null, 2))
}


main();
