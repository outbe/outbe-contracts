import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";

export const METADOSIS_CONTRACT_ADDRESS = "outbe1ztze0tumcmpjre8awusms3t2sjajlxcz34cn6enp5yfw9xf3dqtq7pznn8"
export const NOD_CONTRACT_ADDRESS = "outbe19klnkygcz4l93seh4wysd05f2lvd62r69hpz42cp9vz86xzlgn0qjr64ld"
export const TRIBUTE_CONTRACT_ADDRESS = "outbe18fwugcqcxeadylwm8trsr4vh3seeaem2uj92mmes4far2mk3a4fscdfh2s"

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




