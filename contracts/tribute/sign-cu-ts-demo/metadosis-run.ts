import {promises as fs} from "fs";
import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";
import {CosmWasmClient, SigningCosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {parseCoins} from "@cosmjs/amino";
import {WalletKeyInfo} from "./generate-wallets";


const METADOSIS_CONTRACT_ADDRESS = "outbe12j3gvpmcte38khlkez28wysp3dwrw0gwwss7w5qxg7hzxmk9ku9s463evk"


// Example of how to use the function:
const endpoint = "https://rpc.dev.outbe.net";

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
    let runnerWallet = await runner()
    const [{address}] = await runnerWallet.getAccounts()

    let client = await CosmWasmClient.connect(endpoint);
    let balance = await client.getBalance(address, "unit")
    console.log("Balance: ", balance)
    let height = await client.getHeight()
    console.log("Current Height: ", height)

    let walletClient = await SigningCosmWasmClient.connectWithSigner(endpoint, runnerWallet)

    let current_timestamp = getCurrentUnixTimestamp();
    let current_date = normalize_to_date(current_timestamp);
    console.log("Current timestamp: ", current_timestamp)
    console.log("Current date: ", current_date)

    let tx = await walletClient.execute(address, METADOSIS_CONTRACT_ADDRESS, {
        execute: {
            run_date: "1750982400"
        }
    }, {
        amount: parseCoins("1unit"),
        gas: "50000000",
    })

    console.log(" Executed Metadosis, tx ", tx.transactionHash)


    let runInfo = await client.queryContractSmart(METADOSIS_CONTRACT_ADDRESS, {
        daily_runs: {}
    })
    console.log("runInfo:")
    console.log(JSON.stringify(runInfo, null,  2))
}


function getCurrentUnixTimestamp(): number {
    return (Math.floor(Date.now() / 1000));
}

function normalize_to_date(ts: number): number {
    // 86400 seconds in a day
    let days = Math.floor(ts / 86400);
    return days * 86400;
}


main();
