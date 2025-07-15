import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";

export const METADOSIS_CONTRACT_ADDRESS = "outbe1kp8ygrq6uqycw5ap6z2ysg3348l0sculpj4g0vaa6x0fw2rd3l5sz8fg3t"
export const NOD_CONTRACT_ADDRESS = "outbe1zw2tqjpwd36ajqlsvzf8t9kdauqm29m4pwgswjkn09z3mz68ymwsl32qw3"
export const TRIBUTE_CONTRACT_ADDRESS = "outbe19fk464n6nny0rx8q7atqhqj923wz6dugn6xcdml65rwaat67khcsfr4lwa"
export const TRIBUTE_FACTORY_CONTRACT_ADDRESS = "outbe1k6qcls2jyz0pzlsf0hm5p8w3ete3n949m43xn6ptjemgzh5jrwzqs92x6s"

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




