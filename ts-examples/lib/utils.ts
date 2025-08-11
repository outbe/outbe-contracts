import {WalletKeyInfo} from "../scripts/generate-wallets";
import {promises as fs} from "fs";
import {blake3} from '@noble/hashes/blake3';
import {bytesToHex} from '@noble/hashes/utils';

export function getRandomInt(min: number, max: number): number {
    return Math.floor(Math.random() * (max - min + 1)) + min;
}

export function getCurrentUnixTimestamp(): number {
    return (Math.floor(Date.now() / 1000));
}

export function normalize_to_date(ts: number): number {
    // 86400 seconds in a day
    let days = Math.floor(ts / 86400);
    return days * 86400;
}

export async function readWalletsFromFile(): Promise<WalletKeyInfo[]> {
    const walletsFile = "wallets.json";

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

export function generateTributeDraftId(owner: string, worldwideDay: string): string {
    const hasher = blake3.create();

    hasher.update(new TextEncoder().encode("tribute_draft_id"));
    hasher.update(new TextEncoder().encode(":"));
    hasher.update(new TextEncoder().encode(owner));
    hasher.update(new TextEncoder().encode(":"));
    hasher.update(new TextEncoder().encode(worldwideDay));

    const hashBytes = hasher.digest();
    return bytesToHex(hashBytes);
}
