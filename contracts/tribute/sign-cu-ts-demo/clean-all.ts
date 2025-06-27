import {promises as fs} from "fs";
import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";
import {CosmWasmClient, SigningCosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {parseCoins} from "@cosmjs/amino";
import {WalletKeyInfo} from "./generate-wallets";
import {RPC_ENDPOINT, METADOSIS_CONTRACT_ADDRESS, TRIBUTE_CONTRACT_ADDRESS, NOD_CONTRACT_ADDRESS} from "./consts";


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

    let walletClient = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, runnerWallet)

    let tx = await walletClient.execute(address, TRIBUTE_CONTRACT_ADDRESS, {
        burn_all: {}
    }, {
        amount: parseCoins("1unit"),
        gas: "50000000",
    })

    console.log("Burn all tribute, tx ", tx.transactionHash)

    let tx2 = await walletClient.execute(address, NOD_CONTRACT_ADDRESS, {
        burn_all: {}
    }, {
        amount: parseCoins("1unit"),
        gas: "50000000",
    })

    console.log("Burn all nod, tx ", tx2.transactionHash)

    let tx3 = await walletClient.execute(address, METADOSIS_CONTRACT_ADDRESS, {
        burn_all: {}
    }, {
        amount: parseCoins("1unit"),
        gas: "50000000",
    })

    console.log("Burn metadosis, tx ", tx3.transactionHash)
}

main();
