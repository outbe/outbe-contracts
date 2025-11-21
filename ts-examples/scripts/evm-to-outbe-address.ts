import {bech32} from "bech32";
import * as string_decoder from "node:string_decoder";


async function main() {

// Input Ethereum address
  const ethAddress = "0xA809592D9fC260C3A7D0022aa21e5e890cebBCf8"

  let seiAddress = evmToOutbe(ethAddress);
  let ethAddress2 = outbeToEvm(seiAddress);

  console.log("Origin address:", ethAddress);
  console.log("Outbe address:", seiAddress);
  console.log("Evm address2:", ethAddress2);

}

function evmToOutbe(ethAddress: string): string {
  const hex = ethAddress.replace(/^0x/, '');
  const bytes = Buffer.from(hex, 'hex');
  const words = bech32.toWords(bytes);
  return bech32.encode('outbe', words);
}

function outbeToEvm(bechAddress: string): string {
  let decoded = bech32.decode(bechAddress);
  const words = bech32.fromWords(decoded.words);
  let evmAddress = Buffer.from(words).toString('hex')
  return `0x${evmAddress}`;
}

main().catch(console.error);
