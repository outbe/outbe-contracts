import { generateTributeDraftId } from '../lib/utils';
import bs58 from 'bs58';

describe('generateTributeDraftId', () => {
    const validOwner = '5HpHagT65TDzv1PH4D1wkmPxqHL5vTMzMmPMDqqAqxnwfnXF'; // Valid base58 address
    
    describe('Valid inputs', () => {
        test('should generate consistent IDs for same inputs', () => {
            const days = "2025-01-01";
            const result1 = generateTributeDraftId(validOwner, days);
            const result2 = generateTributeDraftId(validOwner, days);
            
            expect(result1).toBe(result2);
            console.log(result1)
        });

        test('should generate different IDs for different owners', () => {
            const days = "2025-01-01";
            const owner2 = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'; // Different valid base58 address
            
            const result1 = generateTributeDraftId(validOwner, days);
            const result2 = generateTributeDraftId(owner2, days);
            
            expect(result1).not.toBe(result2);
        });

        test('should generate different IDs for different days', () => {
            const result1 = generateTributeDraftId(validOwner, "2025-01-01");
            const result2 = generateTributeDraftId(validOwner, "2025-01-02");
            
            expect(result1).not.toBe(result2);
        });

        test('should generate valid base58 output', () => {
            const result = generateTributeDraftId(validOwner, "2025-01-30");
            
            // Should be a non-empty string
            expect(result).toBeTruthy();
            expect(typeof result).toBe('string');
            
            // Should contain only valid base58 characters
            const base58Regex = /^[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]+$/;
            expect(base58Regex.test(result)).toBe(true);
            
            // Should be decodable as base58
            expect(() => bs58.decode(result)).not.toThrow();
        });

        test('should handle zero days', () => {
            const result = generateTributeDraftId(validOwner, "2025-01-01");
            
            expect(result).toBeTruthy();
            expect(typeof result).toBe('string');
        });

        test('should produce deterministic output', () => {
            const testCases = [
                { owner: validOwner, days: "2025-01-01" },
                { owner: validOwner, days: "2025-01-02" },
                { owner: validOwner, days: "2026-01-01" },
                { owner: validOwner, days: "3025-01-01" }
            ];

            testCases.forEach(({ owner, days }) => {
                const result1 = generateTributeDraftId(owner, days);
                const result2 = generateTributeDraftId(owner, days);
                expect(result1).toBe(result2);
            });
        });
    });

    describe('Output characteristics', () => {
        test('should produce consistent length output for SHA256 hash', () => {
            const results = [
                generateTributeDraftId(validOwner, "2025-01-01"),
                generateTributeDraftId(validOwner, "2026-01-01"),
                generateTributeDraftId(validOwner, "3333-01-01")
            ];

            // All results should be base58 encoded SHA256 hashes
            // SHA256 produces 32 bytes, base58 encoding will vary in length but should be consistent for same input
            results.forEach(result => {
                const decoded = bs58.decode(result);
                expect(decoded.length).toBe(32); // SHA256 hash is always 32 bytes
            });
        });

    });

    describe('Error handling', () => {
        test('should throw error for invalid base58 owner', () => {
            expect(() => generateTributeDraftId('invalid-base58-0OIl', "2025-01-01")).toThrow();
        });

        test('should handle empty owner (produces valid result)', () => {
            // Empty string is valid base58 and decodes to empty array
            const result = generateTributeDraftId('', "2025-01-01");
            expect(result).toBeTruthy();
            expect(typeof result).toBe('string');
        });
    });
});
