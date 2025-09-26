import {getContractAddresses, initClient} from "../lib/clientService";
import {TributeQueryClient} from "../clients/tribute/Tribute.client";
import {RUN_DATE, RUN_DATE_TS} from "../config";
import {CosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {PriceOracleQueryClient} from "../clients/price-oracle/PriceOracle.client";

async function main() {
  const {walletClient} = await initClient()

  let height = await walletClient.getHeight()
  console.log("Current Height:", height)
  console.log("Using RUN_DATE", RUN_DATE, ":", RUN_DATE_TS())

  let coenUsdRate = await queryActualRate(walletClient)
  console.log("Actual 'coen/usd' rate now:", coenUsdRate)

  const tributeContractAddress = await getContractAddresses('TRIBUTE_CONTRACT_ADDRESS');
  const tributeClient = new TributeQueryClient(walletClient, tributeContractAddress)
  let tokensResp = await tributeClient.numTokens();
  console.log("Number of Tribute tokens: ", tokensResp)

  let daily_tokens = await tributeClient.dailyTributes({})
  console.log("Daily Tribute tokens (1 page): ", daily_tokens)
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
