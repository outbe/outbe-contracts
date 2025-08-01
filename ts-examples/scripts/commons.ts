
// SPECIFY HERE DATE FOR RUN FOR METADOSIS
export const RUN_DATE = "2025-07-27";

export function toTimestamp(dateStr: string): number {
    return Math.floor(new Date(dateStr).getTime() / 1000);
}

export function RUN_DATE_TS(): number {
    return toTimestamp(RUN_DATE);
}

