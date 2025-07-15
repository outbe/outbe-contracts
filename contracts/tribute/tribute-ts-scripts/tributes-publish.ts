import {promises as fs} from "fs";
import {CosmWasmClient, SigningCosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {parseCoins} from "@cosmjs/amino";
import {WalletKeyInfo} from "./generate-wallets";
import {
    getRandomInt,
    METADOSIS_CONTRACT_ADDRESS,
    RPC_ENDPOINT,
    runner,
    TRIBUTE_CONTRACT_ADDRESS,
    TRIBUTE_FACTORY_CONTRACT_ADDRESS
} from "./consts";
import {ExecuteInstruction} from "@cosmjs/cosmwasm-stargate/build/signingcosmwasmclient";


const walletsFile = "wallets.json";

export class AllocationResponse {
    public total_allocation: string
    public pool_allocation: string
}

async function readWalletsFromFile(): Promise<WalletKeyInfo[]> {
    try {
        const fileContent = await fs.readFile(walletsFile, 'utf8');
        const wallets: WalletKeyInfo[] = JSON.parse(fileContent);
        console.log(`Successfully loaded ${wallets.length} wallets.`);
        return wallets;
    } catch (error) {
        console.error(`Error reading or parsing ${walletsFile}:`, error);
        return [];
    }
}

async function main() {
    const wallets = await readWalletsFromFile();
    if (wallets.length > 0) {
        console.log("First wallet loaded:", wallets[0]);
    }

    const [runnerWallet, runnerAddress] = await runner()

    let client = await CosmWasmClient.connect(RPC_ENDPOINT);
    let balance = await client.getBalance(runnerAddress, "unit")
    console.log("Balance: ", balance)
    let height = await client.getHeight()
    console.log("Current Height: ", height)

    let tokensResp = await client.queryContractSmart(TRIBUTE_CONTRACT_ADDRESS, {
        num_tokens: {}
    })
    console.log("Number of Tribute Tokens before execution: ", tokensResp)

    console.log("Trying to mint Tributes tx...")

    let walletClient = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, runnerWallet)

    // let current_timestamp = getCurrentUnixTimestamp();
    // let current_date = normalize_to_date(current_timestamp);
    // console.log("Current timestamp: ", current_timestamp)
    // console.log("Current date: ", current_date)

    let allocationResp: AllocationResponse = await client.queryContractSmart(METADOSIS_CONTRACT_ADDRESS, {
        allocation: {}
    })
    let total_alloc = Number(allocationResp.total_allocation)
    let avg_price = Math.floor(total_alloc / wallets.length * 27)
    console.log("Total Allocation: ", BigInt(allocationResp.total_allocation))
    console.log("Pool Allocation: ", BigInt(allocationResp.pool_allocation))
    console.log("avg_price: ", avg_price)

    let instructions: ExecuteInstruction[] = [];
    for (let i = 0; i < wallets.length; i++) {
        let tribute = randomTribute(wallets[i].outbe_address, "2025-07-15", avg_price)
        instructions.push({
                contractAddress: TRIBUTE_FACTORY_CONTRACT_ADDRESS,
                msg: tribute,
            }
        )
    }
    let tx = await walletClient.executeMultiple(runnerAddress, instructions, {
        amount: parseCoins("1unit"),
        gas: "50000000",
    })

    console.log("created Tributes, tx ", tx.transactionHash)

    let r = await client.queryContractSmart(TRIBUTE_CONTRACT_ADDRESS, {
        num_tokens: {}
    })
    console.log("Number of Tribute tokens: ", r)
}

function randomTribute(owner: string, day: string, avgPrice: number): any {
    let uuid_id = require('crypto').randomUUID().toString()
    let tribute_draft_id = require('crypto').createHash('sha256').update(uuid_id).digest('hex');
    let settlement_amount = getRandomInt(avgPrice - 100, avgPrice + 100);
    

    return {
        offer_insecure: {
            tribute_input: {
                tribute_draft_id: tribute_draft_id,
                owner: owner,
                worldwide_day: day,
                settlement_currency: "usd",
                settlement_base_amount: settlement_amount,
                settlement_atto_amount: 0,
                nominal_base_qty: settlement_amount * 2,
                nominal_atto_qty: 0,
                cu_hashes: [tribute_draft_id]
            },
            zk_proof: {
                proof: "",
                public_data: {
                    public_key: "",
                    merkle_root: "",
                },
                verification_key: "",
            }
        }
    }
}

main();
