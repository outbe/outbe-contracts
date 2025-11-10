import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";
import {promises as fs} from "fs";
import {randomBytes} from "crypto";
import {NUMBER_OF_WALLETS} from "../config";
import {publicKeyToAddress} from "viem/accounts";
import {Hex} from "viem";

const walletsFile = "wallets.json";

export class WalletKeyInfo {
  constructor(
    public outbe_address: string,
    public privateKey: string,
    public publicKey: string,
    public evm_address?: string
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

/**
 * Updates existing wallets file with EVM addresses derived from public keys.
 * Associates bech32 and EVM addresses via Public Key using @sei-js/evm.
 */
async function updateWalletsWithEvmAddresses() {
  try {
    console.log(`Reading wallets from ${walletsFile}...`);

    // Read existing wallets
    const fileContent = await fs.readFile(walletsFile, "utf-8");
    const wallets: WalletKeyInfo[] = JSON.parse(fileContent);

    console.log(`Found ${wallets.length} wallets. Deriving EVM addresses from public keys...`);

    // Update each wallet with EVM address derived from public key
    const updatedWallets = wallets.map((wallet) => {

      // Derive EVM address from public key
      const evmAddress = publicKeyToAddress(wallet.publicKey as Hex);

      console.log(`Wallet ${wallet.outbe_address}:`);
      console.log(`  Public Key: ${wallet.publicKey}`);
      console.log(`  EVM Address: ${evmAddress}`);

      return new WalletKeyInfo(
        wallet.outbe_address,
        wallet.privateKey,
        wallet.publicKey,
        evmAddress
      );
    });

    // Write updated wallets back to file
    await fs.writeFile(walletsFile, JSON.stringify(updatedWallets, null, 2));
    console.log(`\nSuccessfully updated ${updatedWallets.length} wallets with EVM addresses in ${walletsFile}`);

  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      console.error(`Error: ${walletsFile} not found. Please run generateWallets first.`);
    } else {
      console.error("Failed to update wallets with EVM addresses:", error);
    }
    throw error;
  }
}

// Uncomment the function you want to run:

// Generate new wallets
// generateWallets().catch(error => {
//   console.error("Failed to generate wallets:", error);
// });

// Update existing wallets with EVM addresses
updateWalletsWithEvmAddresses().catch(error => {
  console.error("Failed to update wallets with EVM addresses:", error);
});
