import {keccak_256} from '@noble/hashes/sha3';
import {secp256k1} from '@noble/curves/secp256k1';
import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";

export class FullAccountInfo {
  constructor(
    public private_key: string,
    public public_key: string,
    public uncompressed_public_key: string,
    public outbe_address: string,
    public evm_address: string
  ) {
  }
}


// Function to generate addresses from a private key.
// See https://docs.sei.io/learn/accounts#deriving-bech32-and-hex-addresses-from-pubkey
export async function generateAddressesFromPrivateKey(privateKeyHex: string): Promise<FullAccountInfo> {
  // 1. Generate Bech32 address
  // Derive the compressed public key from the private key
  const publicKeyBytes = secp256k1.getPublicKey(privateKeyHex, true);
  const wallet = await DirectSecp256k1Wallet.fromKey(Buffer.from(privateKeyHex, "hex"), "outbe");
  const [account] = await wallet.getAccounts();
  const seiAddress = account.address;

  // 2. Generate EVM address
  // Derive the uncompressed public key from the private key
  const publicKeyUncompressed = secp256k1.getPublicKey(privateKeyHex, false);
  // Exclude the first byte and Perform Keccak-256 hashing on the uncompressed public key to derive the Ethereum address
  const keccakHash = keccak_256(publicKeyUncompressed.slice(1));
  const ethAddress = `0x${Buffer.from(keccakHash).subarray(-20).toString('hex')}`;

  return new FullAccountInfo(
    privateKeyHex,
    Buffer.from(publicKeyBytes).toString('hex'),
    Buffer.from(publicKeyUncompressed).toString('hex'),
    seiAddress,
    ethAddress);
}
