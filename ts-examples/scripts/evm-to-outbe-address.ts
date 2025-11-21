import {bech32} from "bech32";
import {publicKeyToAddress} from "viem/accounts";
import {secp256k1} from "@noble/curves/secp256k1";


async function main() {

// Input Ethereum address
  const ethAddress = "0xA809592D9fC260C3A7D0022aa21e5e890cebBCf8"
  // const ethAddress = "0x37B6cA0eF49865e4E1439D62b21aDdB6aF1Bcf25" // this doesnt work!!

  let seiAddress = evmToOutbe(ethAddress);
  let ethAddress2 = outbeToEvm(seiAddress);

  console.log("Origin address:", ethAddress);
  console.log("Outbe address:", seiAddress);
  console.log("Evm after convert:", ethAddress2);

  console.log("From Pubkey compressed:", fromPubkeyCompressed("03a2c0fb241f2459bf3be23eb3a3763fab25396ae8308ce3302c57d79639eacda4"));
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

function fromPubkeyCompressed(pubkey: string): string {
  let pkc = secp256k1.getPublicKey("222da31b7e9a6ed579b0898f201631b6335d2c248a87f42af7a3f7e0c6127397", true)
  let pku = secp256k1.getPublicKey("222da31b7e9a6ed579b0898f201631b6335d2c248a87f42af7a3f7e0c6127397", false)

  let pkUncompressedHex = Buffer.from(pku).toString('hex')
  let pkCompressedHex = Buffer.from(pkc).toString('hex')
  let evmAddress = publicKeyToAddress(`0x${pkUncompressedHex}`);
  let evmAddress2 = publicKeyToAddress(`0x${pkCompressedHex}`);

  console.log("pkUncompressedHex:", pkUncompressedHex, "===", evmAddress);
  console.log("pkCompressedHex:", pkCompressedHex, "===", evmAddress2);

  return evmAddress;
}

main().catch(console.error);
