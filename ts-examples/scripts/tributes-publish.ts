import {ExecuteInstruction} from "@cosmjs/cosmwasm-stargate/build/signingcosmwasmclient";
import {getContractAddresses, initClient} from "../lib/clientService";
import {TributeQueryClient} from "../clients/tribute/Tribute.client";

import {NumTokensResponse} from "../clients/tribute/Tribute.types";
import {RUN_DATE, TX_FEE} from "../config";
import {generateTributeDraftId, getRandomInt, readWalletsFromFile} from "../lib/utils";
import {TributeInputPayload, ZkProof} from "../clients/tribute-factory/TributeFactory.types";
import {TokenAllocatorQueryClient} from "../clients/token-allocator/TokenAllocator.client";
import {TokenAllocatorData} from "../clients/token-allocator/TokenAllocator.types";

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
    let avg_price = Math.floor(total_alloc / wallets.length / 0.08)
    console.log("Daily Allocation: ", BigInt(total_alloc))
    console.log("avg_price: ", avg_price)

    let tbFactoryContractAddress = await getContractAddresses('TRIBUTE_FACTORY_CONTRACT_ADDRESS');

    let instructions: ExecuteInstruction[] = [];
    for (let i = 0; i < wallets.length; i++) {
        let tribute = randomTribute(wallets[i].outbe_address, RUN_DATE, avg_price)
        instructions.push({
                contractAddress: tbFactoryContractAddress,
                msg: tribute,
            }
        )
    }
    let tx = await walletClient.executeMultiple(account.address, instructions, TX_FEE)
    console.log("created Tributes, tx ", tx.transactionHash)

    console.log("Number of Tribute tokens: ", await tributeClient.numTokens())
}

function randomTribute(owner: string, day: string, avgPrice: number): any {
    let uuid_id = require('crypto').randomUUID().toString()
    let cu_hashes = require('crypto').createHash('sha256').update(uuid_id).digest('hex');
    // let nominal_amount = getRandomInt(1, avgPrice * 2);
    // NB: price not average
    let nominal_amount = getRandomInt(90, 400);
    let settlement_amount = Math.floor(nominal_amount * 0.012);
    let tribute_draft_id = generateTributeDraftId(owner, day);
    console.log("Tribute draft id:", tribute_draft_id,
        "settlement_amount:", settlement_amount, "nominal_amount:", nominal_amount)

    let zk_proof: ZkProof = {
        proof: "",
        public_data: {
            public_key: "",
            merkle_root: "",
        },
        verification_key: "",
    }

    let tribute_input: TributeInputPayload = {
        tribute_draft_id: tribute_draft_id,
        owner: owner,
        worldwide_day: day,
        settlement_currency: "usd",
        settlement_base_amount: settlement_amount,
        settlement_atto_amount: 0,
        nominal_base_qty: nominal_amount,
        nominal_atto_qty: 0,
        cu_hashes: [cu_hashes]
    }

    return {
        offer_insecure: {
            tribute_input,
            zk_proof,
            tribute_owner_l1: owner
        }
    }
}

main();
