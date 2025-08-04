# Scripts example for TS

This example shows how run scripts in TypeScript applications.
## How to install

```bash
# install dependencies
npm install
# Generate Clients, Types from contract-Schema using @cosmwasm/ts-codegen
npm run codegen:generate 
```

## How to run:

```shell
npx ts-node scripts/clean-all.ts # Remove all Tributes, Metadosis, Nod
npx ts-node scripts/generate-wallets.ts # Generate Random wallets
npx ts-node scripts/tributes-publish.ts # Generate Tributes
npx ts-node scripts/tributes-info.ts # Tributes Info
npx ts-node scripts/nods-info.ts # Nods info
npx ts-node scripts/metadosis-prepare.ts # Prepare day to run Metadosis
npx ts-node scripts/metadosis-run.ts # Generate Metadosis
npx ts-node scripts/metadosis-info.ts # Metadosis info
npx ts-node scripts/send-native-coins.ts # Send native coins


```
## Environment Variables (.env)

```
PRT_KEY=
```
