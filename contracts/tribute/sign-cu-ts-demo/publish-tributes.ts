import {promises as fs} from "fs";
import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";
import {CosmWasmClient, SigningCosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {parseCoins} from "@cosmjs/amino";


const walletsFile = "wallets.json";

const TRIBUTE_CONTRACT_ADDRESS = "outbe1wjraj7nqnq8s6qchxds4sl0vdhlymf8mn4hqrg4ez9glwpj0lf2sszdrvh"
const METADOSIS_CONTRACT_ADDRESS = "outbe105pes6h7xmnw4zw4slyhcywga5m2m529uqxj25y6gxdpw46rt7jqqctwpe"

export class WalletKeyInfo {
    constructor(
        public outbe_address: string,
        public privateKey: string,
        public publicKey: string
    ) {
    }
}

export class TributeData {
    constructor(
        tribute_id: string,
        owner: string,
        settlement_amount: string,
        settlement_currency: { "cw20": "usdc" },
        worldwide_day: bigint,
    ) {
    }
}

export class MintExtension {
    constructor(
        data: TributeData,
        signature: string,
        public_key: string,
    ) {
    }
}

export class MintPayload {
    constructor(
        /// Unique ID of the NFT
        token_id: string,
        /// The owner of the newly minter NFT
        owner: string,
        extension: MintExtension
    ) {
    }
}

export class MintTribute {
    constructor(
        mint: MintPayload,
    ) {
    }
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

// Example of how to use the function:
const endpoint = "https://rpc.dev.outbe.net";

async function main() {
    const wallets = await readWalletsFromFile();
    if (wallets.length > 0) {
        console.log("First wallet loaded:", wallets[0]);
    }
    let runnerWallet = await runner()
    const [{address}] = await runnerWallet.getAccounts()

    let client = await CosmWasmClient.connect(endpoint);
    let balance = await client.getBalance(address, "unit")
    console.log("Balance: ", balance)
    let height = await client.getHeight()
    console.log("Current Height: ", height)

    let tokensResp = await client.queryContractSmart(TRIBUTE_CONTRACT_ADDRESS, {
        num_tokens: {}
    })
    console.log("Number of Tribute Tokens before execution: ", tokensResp)

    let mintTribute = new MintTribute({
        mint: new MintPayload(
            "1",
            "outbe1zus30tvmrh2qfazxqcvcvl3n22hsll5z43ujcu",
            new MintExtension(
                new TributeData(
                    "1",
                    "outbe1zus30tvmrh2qfazxqcvcvl3n22hsll5z43ujcu",
                    "100",
                    {"cw20": "usdc"},
                    BigInt(1638400000000000000)
                ),
                "signature",
                "public_key"
            )
        )
    })

    console.log("Trying to mint tx...")

    let walletClient = await SigningCosmWasmClient.connectWithSigner(endpoint, runnerWallet)

    let current_timestamp = getCurrentUnixTimestamp();
    let current_date = normalize_to_date(current_timestamp);
    console.log("Current timestamp: ", current_timestamp)
    console.log("Current date: ", current_date)

    let tribute = randomTribute("outbe1e8r7rng6lratxymrakgqtndkqcnenun54s0uaw", current_date.toString())

    let tx = await walletClient.execute(address, TRIBUTE_CONTRACT_ADDRESS, tribute, {
        amount: parseCoins("1unit"),
        gas: "500000",
    })

    console.log("Transaction: ", tx)

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


function randomTribute(owner: string, day: string): any {
    let tribute_id = require('crypto').randomUUID().toString();
    let settlement_amount = getRandomInt(100, 1000).toString();

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
