import {promises as fs} from "fs";
import {randomBytes} from "crypto";
import {NUMBER_OF_WALLETS} from "../config";
import {FullAccountInfo, generateAddressesFromPrivateKey} from "./generate-address";

const walletsFile = "wallets.json";

async function generateWallets() {
  const wallets: FullAccountInfo[] = [];
  console.log(`Generating ${NUMBER_OF_WALLETS} wallets...`);

  for (let i = 0; i < NUMBER_OF_WALLETS; i++) {
    const privateKey = randomBytes(32);
    let walletInfo = await generateAddressesFromPrivateKey(privateKey.toString("hex"));
    wallets.push(walletInfo);
  }

  await fs.writeFile(walletsFile, JSON.stringify(wallets, null, 2));
  console.log(`Successfully generated and saved ${NUMBER_OF_WALLETS} wallets to ${walletsFile}`);
}

async function main() {
  await generateWallets();
}

main().catch(console.error);
