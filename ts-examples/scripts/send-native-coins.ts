import {promises as fs} from "fs";
import {coins} from "@cosmjs/proto-signing";
import {WalletKeyInfo} from "./generate-wallets";
import {TX_FEE} from "../config";
import {initClient} from "../lib/clientService";


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

async function main() {
  const wallets = await readWalletsFromFile();
  if (wallets.length > 0) {
    console.log("First wallet loaded:", wallets[0]);
  }

  const {walletClient, account} = await initClient()

  let balance = await walletClient.getBalance(account.address, "coen")
  console.log("Balance: ", balance)
  let height = await walletClient.getHeight()
  console.log("Current Height: ", height)

  for (let i = 0; i < wallets.length; i++) {
    const result = await walletClient.sendTokens(
      account.address,
      wallets[i].outbe_address,
      coins("1", "coen"),
      TX_FEE
    );
    console.log(i, ": Sent 1 coen to", wallets[i].outbe_address, "tx", result.transactionHash)
  }
}

main();
