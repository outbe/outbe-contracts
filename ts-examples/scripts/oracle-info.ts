import {getContractAddresses, initClient} from "../lib/clientService";
import {PriceOracleQueryClient} from "../clients/price-oracle/PriceOracle.client";

async function main() {
  const {walletClient, account} = await initClient()
  let contractAddress = await getContractAddresses('PRICE_ORACLE_CONTRACT_ADDRESS')
  let queryClient = new PriceOracleQueryClient(walletClient, contractAddress)

  let all_pairs = await queryClient.getAllPairs()
  for (const pair of all_pairs) {
    let price = await queryClient.getLatestPrice(pair);
    console.log("Price for pair:", pair, "is", parseFloat(price.price));
  }
}

main().catch(console.error);
