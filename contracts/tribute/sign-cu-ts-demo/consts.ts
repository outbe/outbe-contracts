import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";

export const METADOSIS_CONTRACT_ADDRESS = "outbe1lqetepd2rx75mn82thw3qn0upkc0ur56cqyh2hzewhc5wndg2s5sylfpyk"
export const NOD_CONTRACT_ADDRESS = "outbe1l3nvn4nc9ftahmr6zjd4frywzpw7ag87kkl93ms37al2zptst6cq9kus08"
export const TRIBUTE_CONTRACT_ADDRESS = "outbe1ke2k44pwv3kugyaetncu8jp4d7cutxue74tvq6scr0p6gm65k9tsp2m7vt"

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




