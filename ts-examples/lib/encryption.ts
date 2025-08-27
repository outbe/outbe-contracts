import {x25519} from '@noble/curves/ed25519';
import {chacha20poly1305} from '@noble/ciphers/chacha';
import {hkdf} from '@noble/hashes/hkdf';
import {sha256} from '@noble/hashes/sha2';
import {randomBytes} from 'crypto';
import bs58 from 'bs58';
import {TributeInputPayload} from "../clients/tribute-factory/TributeFactory.types";

export interface EncryptedData {
    cipher_text: string;
    nonce: string;
    ephemeral_pubkey: string;
}

const DEFAULT_ENCRYPTION_PRIVATE_KEY = 'a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456';

/**
 * Encrypts data using ECDHE with X25519, HKDF, and ChaCha20Poly1305
 * This matches the decryption logic in the smart contract
 */
export function encryptTributeInput(
    data: TributeInputPayload,
    contractPublicKeyBs58: string,
    saltBs58: string
): EncryptedData {
    if (!contractPublicKeyBs58) {
        throw new Error('Contract public key is required');
    }

    // Convert contract public key from Base58 to bytes
    const contractPublicKey = bs58.decode(contractPublicKeyBs58);

    // Use default salt if not provided (for backward compatibility)
    const salt = bs58.decode(saltBs58);

    // Use a fixed private key for encryption
    const ephemeralPrivateKey = Buffer.from(DEFAULT_ENCRYPTION_PRIVATE_KEY, 'hex');
    // Generate ephemeral keypair for a client
    // const ephemeralPrivateKey = randomBytes(32);
    const ephemeralPublicKey = x25519.getPublicKey(ephemeralPrivateKey);

    // Perform ECDH to get shared secret
    const sharedSecret = x25519.getSharedSecret(ephemeralPrivateKey, contractPublicKey);

    // Use HKDF to derive an encryption key from shared secret and salt
    const encryptionKey = hkdf(sha256, sharedSecret, salt, new TextEncoder().encode('tribute-factory-encryption'), 32);

    // Serialize the data to JSON bytes
    const plaintext = Buffer.from(JSON.stringify(data), 'utf8');

    // Generate random nonce (12 bytes for ChaCha20Poly1305)
    const nonce = randomBytes(12);

    // Encrypt using ChaCha20Poly1305 with the derived key
    const cipher = chacha20poly1305(encryptionKey, nonce);
    const ciphertext = cipher.encrypt(plaintext);

    return {
        cipher_text: bs58.encode(ciphertext),
        nonce: bs58.encode(nonce),
        ephemeral_pubkey: bs58.encode(ephemeralPublicKey)
    };
}
