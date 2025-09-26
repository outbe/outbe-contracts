import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";
import {promises as fs} from "fs";
import {randomBytes} from "crypto";
import {NUMBER_OF_WALLETS} from "../config";

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
  console.log(`Generating ${NUMBER_OF_WALLETS} wallets...`);

  for (let i = 0; i < NUMBER_OF_WALLETS; i++) {
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
  console.log(`Successfully generated and saved ${NUMBER_OF_WALLETS} wallets to ${walletsFile}`);
}

generateWallets().catch(error => {
  console.error("Failed to generate wallets:", error);
});
