import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";
import {promises as fs} from "fs";
import {randomBytes} from "crypto";

const numberOfWallets = 200;
const walletsFile = "wallets.json";

export class WalletKeyInfo {
    constructor(
        public outbe_address: string,
        public privateKey: string,
        public publicKey: string
    ) {
    }
}

async function generateWallets() {
    const wallets: WalletKeyInfo[] = [];
    console.log(`Generating ${numberOfWallets} wallets...`);

    for (let i = 0; i < numberOfWallets; i++) {
        const privateKey = randomBytes(32);
        const wallet = await DirectSecp256k1Wallet.fromKey(privateKey, "outbe");
        const [account] = await wallet.getAccounts();

        const walletInfo = new WalletKeyInfo(
            account.address,
            Buffer.from(privateKey).toString("hex"),
            Buffer.from(account.pubkey).toString("hex")
        );
        wallets.push(walletInfo);
    }

    await fs.writeFile(walletsFile, JSON.stringify(wallets, null, 2));
    console.log(`Successfully generated and saved ${numberOfWallets} wallets to ${walletsFile}`);
}

generateWallets().catch(error => {
    console.error("Failed to generate wallets:", error);
});
