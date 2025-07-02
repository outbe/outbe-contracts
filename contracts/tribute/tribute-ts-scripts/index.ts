import * as secp256k1 from 'secp256k1';
import {createHash} from "node:crypto";

const owner = "outbe1zus30tvmrh2qfazxqcvcvl3n22hsll5z43ujcu";
const token_id = "3";
// prepare raw json data (tribute)
let raw_json = {
    token_id,
    owner,
    settlement_value: "100",
    settlement_token: {cw20: "usdc"},
    tribute_date: null,
    hashes: [createHash("sha256").update(new Date().getTime().toString()).digest("hex")],
};

// 0. Init test keys
let private_key = Buffer.from(
    "4236627b5a03b3f2e601141a883ccdb23aeef15c910a0789e4343aad394cbf6d",
    "hex"
);
if (!secp256k1.privateKeyVerify(private_key)) {
    console.log("private key is not valid");
}
const pubKeyCompressed = secp256k1.publicKeyCreate(private_key, true); // 33 bytes
let publicKeyHex = Buffer.from(pubKeyCompressed).toString("hex");

console.log("Public key (compressed, 33 bytes):", publicKeyHex);

// 1. Hash the message
const encoder = new TextEncoder();
const message = encoder.encode(JSON.stringify(raw_json));
const msgHash = createHash("sha256").update(message).digest();
console.log("message hash:", msgHash.toString("hex"));

// Sign the message hash
const {signature, recid} = secp256k1.ecdsaSign(msgHash, private_key);
let signatureHex = Buffer.from(signature).toString("hex");
console.log("Compact signature (64 bytes):", signatureHex);

// Verify the signature
const verified = secp256k1.ecdsaVerify(signature, msgHash, pubKeyCompressed);
console.log("Signature valid?", verified);

// Now you have everything you need to conduct the mint transaction:
let mintTx = {
    mint: {
        token_id,
        owner,
        token_uri: null,
        extension: {
            entity: raw_json,
            signature: signatureHex,
            public_key: publicKeyHex,
        },
    },
};
console.log("\nMint transaction JSON:\n", JSON.stringify(mintTx), "\n");
