import {getContractAddresses, initClient} from "../lib/clientService";
import {PriceOracleClient, PriceOracleQueryClient} from "../clients/price-oracle/PriceOracle.client";
import {TX_FEE} from "../config";

async function main() {
  const {walletClient, account} = await initClient()
  let contractAddress = await getContractAddresses('PRICE_ORACLE_CONTRACT_ADDRESS')
  let client = new PriceOracleClient(walletClient, account.address, contractAddress)
  let queryClient = new PriceOracleQueryClient(walletClient, contractAddress)

  console.log("Current Coen/USDC Rate:", await queryCoenRate(queryClient))
  console.log("Current XAU/USD Rate:", await queryXauRate(queryClient))

  console.log("Updating rates...")

  // Update Coen/USDC rate
  await client.updatePrice({
    price: "0.013",
    token1: {
      native: "coen"
    },
    token2: {
      native: "usdc"
    },
  }, TX_FEE)

  // Update XAU/USD rate
  await client.updatePrice({
    price: "3305.90",
    token1: {
      commodity: "xau"
    },
    token2: {
      fiat: "usd"
    },
  }, TX_FEE)

  console.log("New Coen/USDC Rate:", await queryCoenRate(queryClient))
  console.log("New XAU/USD Rate:", await queryCoenRate(queryClient))
}

export async function queryCoenRate(queryClient: PriceOracleQueryClient): Promise<number> {
  let response = await queryClient.getLatestPrice({
    token1: {
      native: "coen"
    },
    token2: {
      native: "usdc"
    },
  })

  return parseFloat(response.price)
}

export async function queryXauRate(queryClient: PriceOracleQueryClient): Promise<number> {
  let response = await queryClient.getLatestPrice({
    token1: {
      commodity: "xau"
    },
    token2: {
      fiat: "usd"
    },
  })

  return parseFloat(response.price)
}

main();
