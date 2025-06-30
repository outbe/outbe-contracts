import {promises as fs} from "fs";
import {coins, DirectSecp256k1Wallet} from "@cosmjs/proto-signing";
import {CosmWasmClient, SigningCosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {parseCoins} from "@cosmjs/amino";
import {WalletKeyInfo} from "./generate-wallets";
import {RPC_ENDPOINT, METADOSIS_CONTRACT_ADDRESS, TRIBUTE_CONTRACT_ADDRESS} from "./consts";
import {ExecuteInstruction} from "@cosmjs/cosmwasm-stargate/build/signingcosmwasmclient";
import {JsonObject} from "@cosmjs/cosmwasm-stargate/build/modules";
import {Coin} from "@cosmjs/stargate";


const walletsFile = "wallets.json";

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

    let walletClient = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, runnerWallet)

    for (let i = 0; i < wallets.length; i++) {
        const result = await walletClient.sendTokens(
            address,
            wallets[i].outbe_address,
            coins("1000000000000000000", "unit"),
            {
                amount: parseCoins("1unit"),
                gas: "500000",
            }
        );
        console.log(i ,": Sent 1 coin to ", wallets[i].outbe_address, " tx ", result.transactionHash)
    }
}

main();
