import {parseCoins} from "@cosmjs/amino";

export const RPC_ENDPOINT = "https://rpc.dev.outbe.net";
export const CONTRACT_REGISTRY_ADDRESS = 'outbe1pu6e36nugjxv3w2tcvxgld39y5kx3zz6l74dwxyyytx9cz5xwg5s4sknnl'

export const TX_FEE = {
  amount: parseCoins("1unit"),
  gas: "50000000",
}

export const NUMBER_OF_WALLETS = 200;

export const NUMBER_OF_METADOSIS_RUNS=1

// SPECIFY HERE DATE FOR RUN FOR METADOSIS
export const RUN_DATE = "2025-07-27";

export function toTimestamp(dateStr: string): number {
  return Math.floor(new Date(dateStr).getTime() / 1000);
}

export function RUN_DATE_TS(): number {
  return toTimestamp(RUN_DATE);
}

