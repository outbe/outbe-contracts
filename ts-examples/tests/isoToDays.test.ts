import { isoToDays, DateError } from '../lib/utils';

describe('isoToDays', () => {
    describe('Valid dates', () => {
        test('should return 0 for epoch date (2025-01-01)', () => {
            expect(isoToDays('2025-01-01')).toBe(0);
        });

        test('should return 1 for day after epoch (2025-01-02)', () => {
            expect(isoToDays('2025-01-02')).toBe(1);
        });

        test('should return 365 for one year after epoch (2026-01-01)', () => {
            expect(isoToDays('2026-01-01')).toBe(365);
        });

        test('should return 31 for February 1st, 2025', () => {
            expect(isoToDays('2025-02-01')).toBe(31);
        });

        test('should handle leap year correctly (2028-02-29)', () => {
            const days = isoToDays('2028-02-29');
            expect(days).toBeGreaterThan(0);
        });

        test('should handle end of year (2025-12-31)', () => {
            expect(isoToDays('2025-12-31')).toBe(364);
        });
    });

    describe('Error cases', () => {
        test('should throw DateError for date before epoch', () => {
            expect(() => isoToDays('2024-12-31')).toThrow(DateError);
            expect(() => isoToDays('2024-12-31')).toThrow('Date before EPOCH');
        });

        test('should throw DateError for invalid date format', () => {
            expect(() => isoToDays('01-01-2025')).toThrow(DateError);
            expect(() => isoToDays('01-01-2025')).toThrow('Invalid date format');
        });

        test('should throw DateError for invalid date format with slashes', () => {
            expect(() => isoToDays('2025/01/01')).toThrow(DateError);
            expect(() => isoToDays('2025/01/01')).toThrow('Invalid date format');
        });

        test('should throw DateError for completely invalid string', () => {
            expect(() => isoToDays('invalid-date')).toThrow(DateError);
            expect(() => isoToDays('invalid-date')).toThrow('Invalid date format');
        });

        test('should throw DateError for empty string', () => {
            expect(() => isoToDays('')).toThrow(DateError);
            expect(() => isoToDays('')).toThrow('Invalid date format');
        });

        test('should handle invalid date (February 30th) by auto-correcting to March 2nd', () => {
            // JavaScript Date constructor auto-corrects invalid dates
            const result = isoToDays('2025-02-30');
            expect(result).toBe(60); // March 2nd, 2025
        });

        test('should throw DateError for invalid month', () => {
            expect(() => isoToDays('2025-13-01')).toThrow(DateError);
            expect(() => isoToDays('2025-13-01')).toThrow('Invalid date');
        });

        test('should throw DateError for invalid day', () => {
            expect(() => isoToDays('2025-01-32')).toThrow(DateError);
            expect(() => isoToDays('2025-01-32')).toThrow('Invalid date');
        });
    });

    describe('Edge cases', () => {
        test('should handle dates far in the future', () => {
            const days = isoToDays('2030-01-01');
            expect(days).toBe(1826); // 5 years * 365 + 1 leap day (2028)
        });

        test('should be consistent with multiple calls', () => {
            const date = '2025-06-15';
            const result1 = isoToDays(date);
            const result2 = isoToDays(date);
            expect(result1).toBe(result2);
        });

        test('should handle single digit months and days with leading zeros', () => {
            expect(isoToDays('2025-01-01')).toBe(0);
            expect(isoToDays('2025-01-09')).toBe(8);
        });
    });
});
