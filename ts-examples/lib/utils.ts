import {WalletKeyInfo} from "../scripts/generate-wallets";
import {promises as fs} from "fs";
import {sha256} from '@noble/hashes/sha2';
import bs58 from 'bs58';

// Define the EPOCH as January 1, 2025
const EPOCH = new Date('2025-01-01T00:00:00.000Z');

export class DateError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'DateError';
    }
}

export function isoToDays(date: string): number {
    // Parse the ISO date string (YYYY-MM-DD format)
    const dateRegex = /^\d{4}-\d{2}-\d{2}$/;
    if (!dateRegex.test(date)) {
        throw new DateError('Invalid date format');
    }

    const parsedDate = new Date(date + 'T00:00:00.000Z');

    // Check if the date is valid
    if (isNaN(parsedDate.getTime())) {
        throw new DateError('Invalid date');
    }

    // Calculate days since EPOCH
    const timeDiff = parsedDate.getTime() - EPOCH.getTime();
    const daysDiff = Math.floor(timeDiff / (1000 * 60 * 60 * 24));

    // Ensure the result is non-negative
    if (daysDiff < 0) {
        throw new DateError('Date before EPOCH');
    }

    return daysDiff;
}

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

export function generateTributeDraftId(owner: string, wwd: number): string {
    const ownerBytes = bs58.decode(owner);

    // Convert wwd number directly to bytes (little-endian 4-byte representation)
    const daysBytes = new Uint8Array(4);
    const view = new DataView(daysBytes.buffer);
    view.setInt32(0, wwd, true); // true for little-endian

    // Concatenate the inputs
    const combined = new Uint8Array(ownerBytes.length + daysBytes.length);
    combined.set(ownerBytes);
    combined.set(daysBytes, ownerBytes.length);
    const hashBytes = sha256(combined);
    return bs58.encode(hashBytes);
}

