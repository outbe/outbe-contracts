#!/usr/bin/env ts-node

import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import fs from "fs";
import path from "path";
import { v4 as uuidv4 } from "uuid";
import faker from "faker";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

type Denom = { native: string } | { cw20: string };

interface NodEntity {
  nod_id: string;
  settlement_token: Denom;
  symbolic_rate: string;
  vector_rate: string;
  nominal_minor_rate: string;
  issuance_minor_rate: string;
  symbolic_minor_load: string;
  vector_minor_rate: string;
  floor_minor_price: string;
  state: "issued" | "settled";
  address: string;
}

interface SubmitMsg {
  submit: {
    token_id: string;
    owner: string;
    extension: {
      entity: NodEntity;
      created_at: null;
    };
  };
}

async function main() {
  const argv = yargs(hideBin(process.argv))
    .option("from-file", {
      type: "string",
      describe: "Path to JSON file with NodEntity data",
    })
    .option("fake", {
      type: "boolean",
      describe: "Generate fake random NodEntity data",
    })
    .option("owner", {
      type: "string",
      demandOption: true,
      describe: "Owner address of the new Nod NFT",
    })
    .option("token-id", {
      type: "string",
      alias: "tokenId",
      demandOption: true,
      describe: "Unique token ID for the new Nod NFT",
    })
    .option("rpc-url", {
      type: "string",
      describe: "RPC endpoint to connect for broadcasting",
    })
    .option("contract", {
      type: "string",
      describe: "Contract address to execute Submit on",
    })
    .option("mnemonic", {
      type: "string",
      describe: "Mnemonic phrase for signing",
    })
    .option("mnemonic-file", {
      type: "string",
      describe: "Path to file containing the mnemonic phrase",
    })
    .option("prefix", {
      type: "string",
      default: "outbe",
      describe: "Bech32 prefix for addresses",
    })
    .option("memo", {
      type: "string",
      default: "",
      describe: "Memo text for the transaction",
    })
    .conflicts("from-file", "fake")
    .conflicts("mnemonic", "mnemonic-file")
    .help()
    .parseSync();

  let entity: NodEntity;
  if (argv.fake) {
    entity = {
      nod_id: uuidv4(),
      settlement_token: { native: faker.finance.currencyCode().toLowerCase() },
      symbolic_rate: faker.datatype
        .number({ min: 0, max: 1, precision: 0.000000001 })
        .toString(),
      vector_rate: faker.datatype
        .number({ min: 0, max: 1, precision: 0.000000001 })
        .toString(),
      nominal_minor_rate: faker.datatype
        .number({ min: 0, max: 1000000 })
        .toString(),
      issuance_minor_rate: faker.datatype
        .number({ min: 0, max: 1000000 })
        .toString(),
      symbolic_minor_load: faker.datatype
        .number({ min: 0, max: 1000000 })
        .toString(),
      vector_minor_rate: faker.datatype
        .number({ min: 0, max: 1000000 })
        .toString(),
      floor_minor_price: faker.datatype
        .number({ min: 0, max: 1000000 })
        .toString(),
      state: "issued",
      address: faker.finance.ethereumAddress(),
    };
  } else if (argv.fromFile) {
    const file = path.resolve(process.cwd(), argv.fromFile);
    const content = fs.readFileSync(file, "utf-8");
    entity = JSON.parse(content) as NodEntity;
  } else {
    console.error("Please specify either --fake or --from-file");
    process.exit(1);
  }

  const msg: SubmitMsg = {
    submit: {
      token_id: argv.tokenId,
      owner: argv.owner,
      extension: {
        entity,
        created_at: null,
      },
    },
  };

  if (argv.rpcUrl) {
    if (!argv.contract) {
      console.error("Missing --contract to specify the contract address");
      process.exit(1);
    }
    if (!argv.mnemonic && !argv.mnemonicFile) {
      console.error("Missing --mnemonic or --mnemonic-file for signing");
      process.exit(1);
    }

    const mnemonicPhrase = argv.mnemonic
      ? argv.mnemonic
      : fs
          .readFileSync(path.resolve(process.cwd(), argv.mnemonicFile), "utf-8")
          .trim();

    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonicPhrase, {
      prefix: argv.prefix,
    });
    const [{ address }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient.connectWithSigner(
      argv.rpcUrl,
      wallet,
      { prefix: argv.prefix },
    );

    console.log(
      `Broadcasting transaction to ${argv.rpcUrl}, contract ${argv.contract}`,
    );
    const result = await client.execute(
      address,
      argv.contract,
      msg.submit,
      "auto",
      argv.memo,
    );
    console.log("Transaction result:", result);
  } else {
    console.log(JSON.stringify(msg, null, 2));
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
