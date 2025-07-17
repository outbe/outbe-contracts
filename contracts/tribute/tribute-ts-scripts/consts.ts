import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";

export const METADOSIS_CONTRACT_ADDRESS = "outbe1wqfl89ks42lktj797jvt2xujer72xfrzx8fst37c4vcw26r0ednqzlg7xm"
export const NOD_CONTRACT_ADDRESS = "outbe12hklu7weu38mhzjsx6vwwqq37rpwgtzx9jmqdans9rgtqmctxtmshyn538"
export const TRIBUTE_CONTRACT_ADDRESS = "outbe1vpz6ypffhtmxh307w3ftr0umfqtenvuxukjvw8gxeg8ets0d395q76zec7"
export const TRIBUTE_FACTORY_CONTRACT_ADDRESS = "outbe1z46lfj857xgf8wcv6y2879pgnn634ftzeplt0csmj0n45wq6rxvqdn2j9f"

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




