import {NUMBER_OF_METADOSIS_RUNS, TX_FEE} from "../config";
import {getContractAddresses, initClient} from "../lib/clientService";
import {getCurrentUnixTimestamp, normalize_to_date} from "../lib/utils";
import {MetadosisClient} from "../clients/metadosis/Metadosis.client";
import {HistoryResponse, MetadosisInfo, MetadosisInfoResponse} from "../clients/metadosis/Metadosis.types";
import {RUN_DATE_TS} from "./commons";


async function main() {
    const {walletClient, account} = await initClient()

    let balance = await walletClient.getBalance(account.address, "unit")
    console.log("Balance: ", balance)
    let height = await walletClient.getHeight()
    console.log("Current Height: ", height)


    const metadosisContractAddress = await getContractAddresses('METADOSIS_CONTRACT_ADDRESS')
    const metadosisClient = new MetadosisClient(walletClient, account.address, metadosisContractAddress)

    let run_date = RUN_DATE_TS();
    let tx_prepare = await metadosisClient.prepare({runDate: run_date}, TX_FEE)
    console.log(`Prepare Metadosis, timestamp:${run_date} tx: ${tx_prepare.transactionHash}`)

    let info: MetadosisInfoResponse = await metadosisClient.metadosisInfo()
    console.log("info:")
    console.log(JSON.stringify(info, null, 2))
}


main();
