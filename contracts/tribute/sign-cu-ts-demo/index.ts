import * as secp256k1 from 'secp256k1';
import {createHash} from "node:crypto";

// prepare raw json data (tribute)
let raw_json = {
    "token_id": "1",
    "owner": "cosmwasm1j2mmggve9m6fpuahtzvwcrj3rud9cqjz9qva39cekgpk9vprae8s4haddx",
    "consumption_value": "100",
    "nominal_quantity": "100",
    "nominal_currency": "usd",
    "hashes": [
        "872be89dd82bcc6cf949d718f9274a624c927cfc91905f2bbb72fa44c9ea876d"
    ]
}

// 0. Init test keys
let private_key = Buffer.from("4236627b5a03b3f2e601141a883ccdb23aeef15c910a0789e4343aad394cbf6d", 'hex');
if (!secp256k1.privateKeyVerify(private_key)) {
    console.log("private key is not valid")
}
const pubKeyCompressed = secp256k1.publicKeyCreate(private_key, true);  // 33 bytes
let publicKeyHex = Buffer.from(pubKeyCompressed).toString('hex')

console.log('Public key (compressed, 33 bytes):', publicKeyHex);

// 1. Hash the message
const encoder = new TextEncoder();
const message = encoder.encode(JSON.stringify(raw_json));
const msgHash = createHash("sha256").update(message).digest()
console.log("message hash:", msgHash.toString('hex'))

// Sign the message hash
const {signature, recid} = secp256k1.ecdsaSign(msgHash, private_key);
let signatureHex = Buffer.from(signature).toString('hex')
console.log('Compact signature (64 bytes):', signatureHex);

// Verify the signature
const verified = secp256k1.ecdsaVerify(signature, msgHash, pubKeyCompressed);
console.log('Signature valid?', verified);

if (signatureHex != "987327f5e1879d8a4739cad9ce0ef3743e5470a6fd2e6d96e67e87701dbcc81b30c6b978d2c40dd022cb9514416ff911ecbe26e9d9d7e726ce82bebf1d41a258") {
    console.error("wrong signature")
}

// Now you have everything you need to conduct the mint transaction:
let mintTx = {
    mint: {
        token_id: "1",
        owner: "cosmwasm1j2mmggve9m6fpuahtzvwcrj3rud9cqjz9qva39cekgpk9vprae8s4haddx",
        extension: {
            entity: raw_json,
            vector: 1,
            signature: signatureHex,
            public_key: publicKeyHex,
        },
    }
}
