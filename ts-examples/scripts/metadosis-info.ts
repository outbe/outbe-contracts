import {getContractAddresses, initClient} from "../lib/clientService";
import {MetadosisQueryClient} from "../clients/metadosis/Metadosis.client";
import {HistoryResponse, MetadosisInfoResponse} from "../clients/metadosis/Metadosis.types";
import {NodQueryClient} from "../clients/nod/Nod.client";

async function main() {
    const {walletClient} = await initClient()

    let height = await walletClient.getHeight()
    console.log("Current Height: ", height)

    console.log( await getContractAddresses());
    const metadosisContractAddress = await getContractAddresses('METADOSIS_CONTRACT_ADDRESS')
    const metadosisClient = new MetadosisQueryClient(walletClient, metadosisContractAddress)
    let info: MetadosisInfoResponse = await metadosisClient.metadosisInfo()
    console.log("info: ")
    console.log(JSON.stringify(info, null, 2))

    let history: HistoryResponse = await metadosisClient.history()
    console.log("")
    console.log("history:")
    console.log(JSON.stringify(history, null, 2))

    console.log("")
    const tributeClient = new NodQueryClient(walletClient, await getContractAddresses('TRIBUTE_CONTRACT_ADDRESS'))
    console.log("Number of Tribute tokens: ", await tributeClient.numTokens())

    const nodContractAddress = await getContractAddresses('NOD_CONTRACT_ADDRESS')
    const nodClient = new NodQueryClient(walletClient, nodContractAddress)
    console.log("Number of Nod tokens: ", await nodClient.numTokens())
}


main();
