import {promises as fs} from "fs";
import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";
import {CosmWasmClient, SigningCosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {parseCoins} from "@cosmjs/amino";
import {WalletKeyInfo} from "./generate-wallets";
import {
    RPC_ENDPOINT,
    METADOSIS_CONTRACT_ADDRESS,
    TRIBUTE_CONTRACT_ADDRESS,
    NOD_CONTRACT_ADDRESS,
    runner
} from "./consts";


async function main() {
    const [runnerWallet, runnerAddress] = await runner()

    let walletClient = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, runnerWallet)

    let tx = await walletClient.execute(runnerAddress, TRIBUTE_CONTRACT_ADDRESS, {
        burn_all: {}
    }, {
        amount: parseCoins("1unit"),
        gas: "50000000",
    })

    console.log("Burn all tribute, tx ", tx.transactionHash)

    let tx2 = await walletClient.execute(runnerAddress, NOD_CONTRACT_ADDRESS, {
        burn_all: {}
    }, {
        amount: parseCoins("1unit"),
        gas: "50000000",
    })

    console.log("Burn all nod, tx ", tx2.transactionHash)

    let tx3 = await walletClient.execute(runnerAddress, METADOSIS_CONTRACT_ADDRESS, {
        burn_all: {}
    }, {
        amount: parseCoins("1unit"),
        gas: "50000000",
    })

    console.log("Burn metadosis, tx ", tx3.transactionHash)
}

main();
