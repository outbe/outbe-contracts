import {getContractAddresses, initClient} from "../lib/clientService";

async function main() {
  const {walletClient} = await initClient()

  let height = await walletClient.getHeight()
  console.log("Current Height:", height)

  let result = await walletClient.searchTx([
    {
      key: "message.sender",
      value: "outbe10p4p27fccqm2hrxqvzhcny7xvatg63576pvxc8",
    },
    {
      key: "execute._contract_address",
      value: "outbe142pyxqfyshpq4ftsj5dkl72ylyhawncceh35v7ghrmqtg6amsmqsezkwx6",
    },
  ]);

  console.log(result);
}

main();
