import {parseCoins} from "@cosmjs/amino";

export const RPC_ENDPOINT = "https://rpc.p.outbe.net";
export const CONTRACT_REGISTRY_ADDRESS = 'outbe18cszlvm6pze0x9sz32qnjq4vtd45xehqs8dq7cwy8yhq35wfnn3qvpnjf9'

export const TX_FEE = {
  amount: parseCoins("1350000unit"),
  gas: "135000000",
}

export const NUMBER_OF_WALLETS = 200;

export const NUMBER_OF_METADOSIS_RUNS = 1

// SPECIFY HERE DATE FOR RUN FOR METADOSIS
export const RUN_DATE = 20251011;

export const AGENTS = [
  {
    address: "outbe1xc2tflxk3d0yqejmc8np4xaxrpgalgf7fr50vt",
    name: "NRA Agent 1",
    email: "nra_1_main@proton.me",
    agent_type: "nra"

  },
  {
    address: "outbe19dew74cngq3dk3y7rep5p0jp7fuphtt4ldjwad",
    name: "NRA Agent 2",
    email: "nra_2_main@proton.me",
    agent_type: "nra"

  },
  {
    address: "outbe1e8r7rng6lratxymrakgqtndkqcnenun54s0uaw",
    name: "NRA Alya",
    email: "xiaoxiongmao88@proton.me",
    agent_type: "nra"

  },

  {
    address: "outbe1hs9uvjnw7p9xn7pldngfej78ua9t5ph24g0jz0",
    name: "CRA Agent 1",
    email: "swoo@swoo.com",
    agent_type: "cra"

  }
];

