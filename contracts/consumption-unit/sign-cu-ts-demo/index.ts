import * as secp256k1 from 'secp256k1';
import {createHash} from "node:crypto";

// prepare raw json data (consumption unit)
let raw_json = {
    "token_id": "1",
    "owner": "cosmwasm1j2mmggve9m6fpuahtzvwcrj3rud9cqjz9qva39cekgpk9vprae8s4haddx",
    "consumption_value": "100",
    "nominal_quantity": "100",
    "nominal_currency": "usd",
    "commitment_tier": 1,
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

if (signatureHex != "dbc7c4d857beea592131a673f5970add886f26a1d1b9cd4fbc68aed82a5b8a342d07d7acb26a12acaacc5cad785534a814c38f67ddacdaf0693943b1aa78fa85") {
    console.error("wrong signature")
}

// Now you have everything you need to conduct the mint transaction:
let mintTx = {
    mint: {
        token_id: "1",
        owner: "cosmwasm1j2mmggve9m6fpuahtzvwcrj3rud9cqjz9qva39cekgpk9vprae8s4haddx",
        extension: {
            entity: raw_json,
            signature: signatureHex,
            public_key: publicKeyHex,
        },
    }
}
