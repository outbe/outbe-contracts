import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";

export const METADOSIS_CONTRACT_ADDRESS = "outbe15q49eke6jt2c4r6apd5axf6qaqjnm6da249trtv5q0tnaudm5jhs6gz3t8"
export const NOD_CONTRACT_ADDRESS = "outbe16nrccj8vy5lkla4lnzuumkcmq9gwmw93huty7ad4mp2artm4raxqa204zk"
export const TRIBUTE_CONTRACT_ADDRESS = "outbe12wl9h6ud8w6uamvzzwefxulwnc56xjwzgy876dqt8tmrvprjdnjsxhsr92"

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




