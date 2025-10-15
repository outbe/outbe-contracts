import {ExecuteInstruction} from "@cosmjs/cosmwasm-stargate/build/signingcosmwasmclient";
import {getContractAddresses, initClient} from "../lib/clientService";
import {TributeQueryClient} from "../clients/tribute/Tribute.client";

import {NumTokensResponse} from "../clients/tribute/Tribute.types";
import {RUN_DATE, TX_FEE} from "../config";
import {generateTributeDraftId, getRandomInt, readWalletsFromFile} from "../lib/utils";
import {EncryptionInfoResponse, TributeInputPayload} from "../clients/tribute-factory/TributeFactory.types";
import {TokenAllocatorQueryClient} from "../clients/token-allocator/TokenAllocator.client";
import {TokenAllocatorData} from "../clients/token-allocator/TokenAllocator.types";
import {CosmWasmClient, JsonObject} from "@cosmjs/cosmwasm-stargate";
import {PriceOracleQueryClient} from "../clients/price-oracle/PriceOracle.client";
import {encryptTributeInput} from "../lib/encryption";
import bs58 from "bs58";
import {TributeFactoryQueryClient} from "../clients/tribute-factory/TributeFactory.client";

async function main() {
  const wallets = await readWalletsFromFile();
  if (wallets.length > 0) {
    console.log("First wallet loaded:", wallets[0]);
  }

  const {walletClient, account} = await initClient()

  let balance = await walletClient.getBalance(account.address, "unit")
  console.log("Balance: ", balance)
  let height = await walletClient.getHeight()
  console.log("Current Height: ", height)

  let tributeContractAddress = await getContractAddresses('TRIBUTE_CONTRACT_ADDRESS');
  let tributeClient = new TributeQueryClient(walletClient, tributeContractAddress)

  let tokensResp: NumTokensResponse = await tributeClient.numTokens();
  console.log("Number of Tribute Tokens before execution: ", tokensResp)

  console.log("Trying to mint Tributes tx...")

  let allocatorContractAddress = await getContractAddresses('TOKEN_ALLOCATOR_CONTRACT_ADDRESS');
  let allocatorClient = new TokenAllocatorQueryClient(walletClient, allocatorContractAddress)

  let allocationResp: TokenAllocatorData = await allocatorClient.dailyAllocation()
  let total_alloc = Number(allocationResp.amount)
  console.log("Daily Allocation: ", BigInt(total_alloc))

  let tbFactoryContractAddress = await getContractAddresses('TRIBUTE_FACTORY_CONTRACT_ADDRESS');

  let coenUsdcRate = await queryActualRate(walletClient)

  let encryptionInfo = await queryEncryptionInfo(walletClient, tbFactoryContractAddress)

  let instructions: ExecuteInstruction[] = [];
  for (let i = 0; i < wallets.length; i++) {
    let tribute = randomTribute(wallets[i].outbe_address, RUN_DATE, coenUsdcRate)

    let msg = offerTribute(tribute, encryptionInfo)
    // let msg = offerInsecureTribute(tribute)

    instructions.push({
        contractAddress: tbFactoryContractAddress,
        msg: msg
      }
    )
  }
  let tx = await walletClient.executeMultiple(account.address, instructions, TX_FEE)
  console.log(instructions.length, "Tributes created, tx ", tx.transactionHash)

  console.log("Number of Tribute tokens: ", await tributeClient.numTokens())
}

function offerTribute(tribute: TributeInputPayload, encryption: EncryptionInfoResponse): JsonObject {
  const encryptedData = encryptTributeInput(tribute, encryption.public_key, encryption.salt);
  let owner = new TextDecoder().decode(bs58.decode(tribute.owner));
  return {
    offer: {
      cipher_text: encryptedData.cipher_text,
      nonce: encryptedData.nonce,
      ephemeral_pubkey: encryptedData.ephemeral_pubkey,
      zk_proof: {
        proof: "",
        public_data: {
          public_key: "",
          merkle_root: "",
        },
        verification_key: "",
      },
      tribute_owner_l1: owner,
    }
  }
}

function offerInsecureTribute(tribute: TributeInputPayload): JsonObject {
  let owner = new TextDecoder().decode(bs58.decode(tribute.owner));
  return {
    offer_insecure: {
      tribute_input: tribute,
      zk_proof: {
        proof: "",
        public_data: {
          public_key: "",
          merkle_root: "",
        },
        verification_key: "",
      },
      tribute_owner_l1: owner,
    }
  }
}

function randomTribute(owner: string, wwd: number, coenUsdcRate: number): TributeInputPayload {
  let uuid_id = require('crypto').randomUUID().toString()
  let cu_hashes = bs58.encode(new TextEncoder().encode(uuid_id));
  let settlement_amount = getRandomInt(90, 400);
  let nominal_amount = Math.floor(settlement_amount / coenUsdcRate);
  let owner_bs58 = bs58.encode(new TextEncoder().encode(owner));
  let tribute_draft_id = generateTributeDraftId(owner_bs58, wwd);
  console.log("Tribute draft id:", tribute_draft_id,
    "settlement_amount:", settlement_amount, "nominal_amount:", nominal_amount)

  return {
    tribute_draft_id: tribute_draft_id,
    owner: owner_bs58,
    worldwide_day: wwd,
    settlement_currency: 840,
    settlement_base_amount: settlement_amount.toString(),
    settlement_atto_amount: "0",
    nominal_base_amount: nominal_amount.toString(),
    nominal_atto_amount: "0",
    cu_hashes: [cu_hashes]
  };
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


export async function queryEncryptionInfo(walletClient: CosmWasmClient, address: string): Promise<EncryptionInfoResponse> {
  let client = new TributeFactoryQueryClient(walletClient, address)
  return await client.encryptionInfo()
}

main();
