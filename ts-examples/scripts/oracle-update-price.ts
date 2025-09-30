import {getContractAddresses, initClient} from "../lib/clientService";
import {CosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {PriceOracleClient, PriceOracleQueryClient} from "../clients/price-oracle/PriceOracle.client";
import {TX_FEE} from "../config";

async function main() {
  const {walletClient, account} = await initClient()

  console.log("Current Rate:", await queryActualRate(walletClient))

  let client = new PriceOracleClient(walletClient, account.address, await getContractAddresses('PRICE_ORACLE_CONTRACT_ADDRESS'))

  await client.updatePrice({
    price: "0.013",
    token1: {
      native: "coen"
    },
    token2: {
      native: "usdc"
    },
  }, TX_FEE)

  console.log("New Rate:", await queryActualRate(walletClient))
}

export async function queryActualRate(walletClient: CosmWasmClient): Promise<number> {
  let address = await getContractAddresses('PRICE_ORACLE_CONTRACT_ADDRESS')
  let client = new PriceOracleQueryClient(walletClient, address)
  let response = await client.getLatestPrice({
    token1: {
      native: "coen"
    },
    token2: {
      native: "usdc"
    },
  })

  return parseFloat(response.price)
}

main();
