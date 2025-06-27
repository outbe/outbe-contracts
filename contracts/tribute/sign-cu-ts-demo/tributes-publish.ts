import {promises as fs} from "fs";
import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";
import {CosmWasmClient, SigningCosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {parseCoins} from "@cosmjs/amino";
import {WalletKeyInfo} from "./generate-wallets";
import {RPC_ENDPOINT, METADOSIS_CONTRACT_ADDRESS, TRIBUTE_CONTRACT_ADDRESS} from "./consts";
import {ExecuteInstruction} from "@cosmjs/cosmwasm-stargate/build/signingcosmwasmclient";
import {JsonObject} from "@cosmjs/cosmwasm-stargate/build/modules";
import {Coin} from "@cosmjs/stargate";


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

async function runner(): Promise<DirectSecp256k1Wallet> {
    let private_key = Buffer.from(
        "4236627b5a03b3f2e601141a883ccdb23aeef15c910a0789e4343aad394cbf6d",
        "hex"
    );
    let wallet = await DirectSecp256k1Wallet.fromKey(private_key, "outbe");
    const [{address}] = await wallet.getAccounts();

    console.log("Using runner address ", address);

    return wallet;
}

async function main() {
    const wallets = await readWalletsFromFile();
    if (wallets.length > 0) {
        console.log("First wallet loaded:", wallets[0]);
    }
    let runnerWallet = await runner()
    const [{address}] = await runnerWallet.getAccounts()

    let client = await CosmWasmClient.connect(RPC_ENDPOINT);
    let balance = await client.getBalance(address, "unit")
    console.log("Balance: ", balance)
    let height = await client.getHeight()
    console.log("Current Height: ", height)

    let tokensResp = await client.queryContractSmart(TRIBUTE_CONTRACT_ADDRESS, {
        num_tokens: {}
    })
    console.log("Number of Tribute Tokens before execution: ", tokensResp)

    console.log("Trying to mint tx...")

    let walletClient = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, runnerWallet)

    // let current_timestamp = getCurrentUnixTimestamp();
    // let current_date = normalize_to_date(current_timestamp);
    // console.log("Current timestamp: ", current_timestamp)
    // console.log("Current date: ", current_date)

    let allocationResp: AllocationResponse = await client.queryContractSmart(METADOSIS_CONTRACT_ADDRESS, {
        allocation: {}
    })
    let total_alloc = Number(allocationResp.total_allocation)
    let avg_price = Math.floor(total_alloc / wallets.length * 7)
    console.log("Total Allocation: ", BigInt(allocationResp.total_allocation))
    console.log("Pool Allocation: ", BigInt(allocationResp.pool_allocation))
    console.log("avg_price: ", avg_price)

    let instructions: ExecuteInstruction[] = [];
    for (let i = 0; i < wallets.length; i++) {
        let tribute = randomTribute(wallets[i].outbe_address, "1751032239445134172", avg_price)
        instructions.push({
                contractAddress: TRIBUTE_CONTRACT_ADDRESS,
                msg: tribute,
            }
        )
    }
    let tx = await walletClient.executeMultiple(address, instructions, {
        amount: parseCoins("1unit"),
        gas: "50000000",
    })

    console.log("created Tributes, tx ", tx.transactionHash)

    let r = await client.queryContractSmart(TRIBUTE_CONTRACT_ADDRESS, {
        num_tokens: {}
    })
    console.log("Number of Tribute tokens: ", r)
}

function getRandomInt(min: number, max: number): number {
    return Math.floor(Math.random() * (max - min + 1)) + min;
}

function getCurrentUnixTimestamp(): number {
    return (Math.floor(Date.now() / 1000));
}

function normalize_to_date(ts: number): number {
    // 86400 seconds in a day
    let days = Math.floor(ts / 86400);
    return days * 86400;
}


function randomTribute(owner: string, day: string, avgPrice: number): any {
    let tribute_id = require('crypto').randomUUID().toString();
    let settlement_amount = getRandomInt(avgPrice - 1000000, avgPrice + 1000000).toString();

    return {
        mint: {
            token_id: tribute_id,
            owner: owner,
            token_uri: null,
            extension: {
                data: {
                    tribute_id: tribute_id,
                    owner: owner,
                    settlement_amount: settlement_amount,
                    settlement_currency: {"cw20": "usdc"},
                    worldwide_day: day,
                },
                signature: "b4f0e146c41699ffe66c144402ea53de9b65f354b8cfcaf884f8b1c33e39726a3c39658859c3d57df77ed62b071f44f9de7b6005e6f7c7721bb39242f554f042",
                public_key: "02c21cb8a373fb63ee91d6133edcd18aefd7fa804adb2a0a55b1cb2f6f8aef068d",
            },
        }
    }
}

main();
