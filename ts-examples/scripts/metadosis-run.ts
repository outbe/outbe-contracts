import {NUMBER_OF_METADOSIS_RUNS, RUN_DATE_TS, TX_FEE} from "../config";
import {getContractAddresses, initClient} from "../lib/clientService";
import {MetadosisClient} from "../clients/metadosis/Metadosis.client";
import {HistoryResponse, MetadosisInfoResponse} from "../clients/metadosis/Metadosis.types";

async function main() {
    const {walletClient, account} = await initClient()

    let balance = await walletClient.getBalance(account.address, "unit")
    console.log("Balance: ", balance)
    let height = await walletClient.getHeight()
    console.log("Current Height: ", height)

    const metadosisContractAddress = await getContractAddresses('METADOSIS_CONTRACT_ADDRESS')
    const metadosisClient = new MetadosisClient(walletClient, account.address, metadosisContractAddress)

    for (let i = 1; i <= NUMBER_OF_METADOSIS_RUNS; i++) {
        let tx = await metadosisClient.execute({runDate: RUN_DATE_TS()}, TX_FEE)
        console.log(i + ": Executed Metadosis, tx ", tx.transactionHash)
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


main();
