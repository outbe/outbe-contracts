import codegen from "@cosmwasm/ts-codegen";
import fs from "fs";
import path from "path";

const CONTRACTS_ROOT = path.resolve(__dirname, "../contracts");
const CLIENTS_ROOT = path.resolve(__dirname, "./clients");

async function main() {
  const contractsDirs = fs.readdirSync(CONTRACTS_ROOT, {withFileTypes: true})
    .filter(dirent => dirent.isDirectory())
    .map(dirent => dirent.name);

  for (const contract of contractsDirs) {
    const schemaDir = path.join(CONTRACTS_ROOT, contract, "schema");
    if (!fs.existsSync(schemaDir)) continue;

    const outDir = path.join(CLIENTS_ROOT, contract);
    console.log(`🔧 Generating client && types for ${contract}...`);

    await codegen({
      contracts: [
        {
          name: contract,
          dir: schemaDir,
        },
      ],
      outPath: outDir,
      options: {
        types: {enabled: true},
        client: {enabled: true},
        bundle: {
          enabled: false,
        },
      },

    });

  }


  console.log("✅ All clients generated");
}

main();
