import {
  getContractAddresses,
  initClient,
} from "../lib/clientService";
import {TributeClient} from "../clients/tribute/Tribute.client";
import {ExecuteResult} from "@cosmjs/cosmwasm-stargate";
import {NodClient} from "../clients/nod/Nod.client";
import {MetadosisClient} from "../clients/metadosis/Metadosis.client";
import {TX_FEE} from "../config";


async function main() {
  const {walletClient, account} = await initClient()

  let tributeContractAddress = await getContractAddresses('TRIBUTE_CONTRACT_ADDRESS')
  let tributeClient = new TributeClient(walletClient, account.address, tributeContractAddress)
  let tx: ExecuteResult = await tributeClient.burnAll(TX_FEE)
  console.log("Burn all tribute, tx ", tx.transactionHash)


  let nodContractAddress = await getContractAddresses('NOD_CONTRACT_ADDRESS')
  let nodClient = new NodClient(walletClient, account.address, nodContractAddress)
  let tx2: ExecuteResult = await nodClient.burnAll(TX_FEE)
  console.log("Burn all nod, tx ", tx2.transactionHash)


  let metadosisContractAddress = await getContractAddresses('METADOSIS_CONTRACT_ADDRESS')
  let metadosisClient = new MetadosisClient(walletClient, account.address, metadosisContractAddress)
  let tx3: ExecuteResult = await metadosisClient.burnAll(TX_FEE)
  console.log("Burn metadosis, tx ", tx3.transactionHash)
}

main();
