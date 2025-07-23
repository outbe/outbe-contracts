import {parseCoins} from "@cosmjs/amino";

export const RPC_ENDPOINT = "https://rpc.dev.outbe.net";
export const CONTRACT_REGISTRY_ADDRESS = 'outbe1pu6e36nugjxv3w2tcvxgld39y5kx3zz6l74dwxyyytx9cz5xwg5s4sknnl'

export const TX_FEE = {
  amount: parseCoins("1unit"),
  gas: "50000000",
}

export const NUMBER_OF_WALLETS = 5;

export const NUMBER_OF_METADOSIS_RUNS=22

