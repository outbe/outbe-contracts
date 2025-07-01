import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";

export const METADOSIS_CONTRACT_ADDRESS = "outbe15uar6jczl530gnlx8pqkyja8xwsrtlh6jsst4zmf8nl5sgace99sg0q3ze"
export const NOD_CONTRACT_ADDRESS = "outbe1ugvp50czec65v782k664pk8t6kxryn7yv9halgz82256masklm9sczmetw"
export const TRIBUTE_CONTRACT_ADDRESS = "outbe1pfy82cw3s033f7dng65s9jdj89273de6dgae7036kgfsrhg8acxq5r6mw8"

export const RPC_ENDPOINT = "https://rpc.dev.outbe.net";


export async function runner(): Promise<[DirectSecp256k1Wallet, string]> {
    let private_key = Buffer.from(
        "4236627b5a03b3f2e601141a883ccdb23aeef15c910a0789e4343aad394cbf6d",
        "hex"
    );
    let wallet = await DirectSecp256k1Wallet.fromKey(private_key, "outbe");
    const [{address}] = await wallet.getAccounts();

    console.log("Using runner address ", address);

    return [wallet, address];
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




