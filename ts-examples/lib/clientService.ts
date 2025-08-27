import {SigningCosmWasmClient} from "@cosmjs/cosmwasm-stargate";
import {AccountData} from "@cosmjs/proto-signing/build/signer";
import {DirectSecp256k1Wallet} from "@cosmjs/proto-signing";
import {CONTRACT_REGISTRY_ADDRESS, RPC_ENDPOINT} from "../config";
import {config} from "dotenv";

config();

let walletClient: SigningCosmWasmClient;
let account: AccountData;


export async function initClient(): Promise<{ walletClient: SigningCosmWasmClient, account: AccountData }> {
    if (walletClient && account) return {walletClient, account};
    let private_key = Buffer.from(
        process.env.PRT_KEY!,
        "hex"
    );

    const wallet = await DirectSecp256k1Wallet.fromKey(
        private_key,
        "outbe"
    );
    walletClient = await SigningCosmWasmClient.connectWithSigner(
        RPC_ENDPOINT,
        wallet
    );
    [account] = await wallet.getAccounts();
    console.log("Using runner address ", account.address);

    return {walletClient, account}
}

export async function getContractAddresses(name?: string) {
    const {walletClient} = await initClient();

    const contractResp = await walletClient.queryContractSmart(CONTRACT_REGISTRY_ADDRESS!, {
        get_deployment: {commit_id: "afb0cc1a39085c813035aa35604d889207e4d38d"}, // null means latest
    })
        .catch((error: any) => {
            console.error(error);
            return undefined;
        });


    if (!contractResp) return undefined;

    const allAddresses = contractResp.deployment.contracts.reduce((acc: Record<string, string>, contract: any) => {
        acc[contract.name] = contract.address;
        return acc;
    }, {});

    if (name) {
        return allAddresses[name];
    }

    return allAddresses;
}



