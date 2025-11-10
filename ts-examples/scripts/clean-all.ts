import {getContractAddresses, initClient,} from "../lib/clientService";
import {TributeClient} from "../clients/tribute/Tribute.client";
import {ExecuteResult, SigningCosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {NodClient} from "../clients/nod/Nod.client";
import {MetadosisClient} from "../clients/metadosis/Metadosis.client";
import {TX_FEE} from "../config";
import {TributeFactoryClient} from "../clients/tribute-factory/TributeFactory.client";


async function main() {
  const {walletClient, account} = await initClient()

  await burnTributes(walletClient, account.address)
  await burnTributeFactory(walletClient, account.address)
  await burnNods(walletClient, account.address)
  await burnMetadosis(walletClient, account.address)

}

async function burnTributes(walletClient: SigningCosmWasmClient, address: string) {
  let tributeContractAddress = await getContractAddresses('TRIBUTE_CONTRACT_ADDRESS')
  let tributeClient = new TributeClient(walletClient, address, tributeContractAddress)
  do {
    let tx: ExecuteResult = await tributeClient.burnAll({batchSize: 50}, TX_FEE)
    console.log("Burned 50 tributes ..., tx", tx.transactionHash)
  } while (await tributeClient.numTokens().then(r => r.count > 0))
  console.log("Burning tributes done ✅")
}

async function burnTributeFactory(walletClient: SigningCosmWasmClient, address: string) {
  let tributeFactoryContractAddress = await getContractAddresses('TRIBUTE_FACTORY_CONTRACT_ADDRESS')
  let tributeFactoryClient = new TributeFactoryClient(walletClient, address, tributeFactoryContractAddress)
  let tx1: ExecuteResult = await tributeFactoryClient.burnAll(TX_FEE)
  console.log("Burn tribute factory done ✅, tx", tx1.transactionHash)
}

async function burnNods(walletClient: SigningCosmWasmClient, address: string) {
  let contractAddress = await getContractAddresses('NOD_CONTRACT_ADDRESS')
  let client = new NodClient(walletClient, address, contractAddress)
  do {
    let tx: ExecuteResult = await client.burnAll({batchSize: 50}, TX_FEE)
    console.log("Burned 50 Nods ..., tx", tx.transactionHash)
  } while (await client.numTokens().then(r => r.count > 0))
  console.log("Burning Nods done ✅")
}

async function burnMetadosis(walletClient: SigningCosmWasmClient, address: string) {
  let metadosisContractAddress = await getContractAddresses('METADOSIS_CONTRACT_ADDRESS')
  let metadosisClient = new MetadosisClient(walletClient, address, metadosisContractAddress)
  let tx: ExecuteResult = await metadosisClient.burnAll(TX_FEE)
  console.log("Burn metadosis done ✅, tx", tx.transactionHash)
}

main().catch(console.error);
