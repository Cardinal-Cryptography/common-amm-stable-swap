import fs from "fs";

import { WsProvider, Keyring, ApiPromise } from "@polkadot/api";

import StablePoolConstructors from "../typed_contracts/constructors/stable_pool_contract";

const RPC_URL = "wss://ws.test.azero.dev";
const DEPLOYER_SEED = "section token crazy better fitness reflect similar witness friend east monitor police";

const tUSDC = "5DEeMqcbWEYD2X9EhMraSxndYYzfWHA6pCsFQkYAPX4FLWLd";
const tUSDT = "5CU5WNiYUkTfFZxJYR7AySfUqvEcHj5zr526CuC7iiVCHPfa";
const wAZERO = "5EFDb7mKbougLtr5dnwd5KDfZ3wK55JPGPLiryKq4uRMPR46";
const sAZERO = "5G1AaZHj1Dm8Xo1jsscxNaaqzrUhtxuW4gsHrWd4wJ4HdST2";
const rateProvider = "5FQtz68upJqkr4zKwV9MWAWt92NDh1iSHHo9u14SermExWFH";

/**
 * Stores addresses in a JSON file.
 * @param addresses - The addresses to store.
*/
export function storeAddresses(addresses: {[key: string]: [value: string]}): void {
  fs.writeFileSync(
    __dirname + "/../addresses.json",
    JSON.stringify(addresses, null, 2)
  );
}

async function main(): Promise<void> {

  const wsProvider = new WsProvider(RPC_URL);
  const keyring = new Keyring({ type: "sr25519" });

  const api = await ApiPromise.create({ provider: wsProvider });
  const deployer = keyring.addFromMnemonic(DEPLOYER_SEED);
  console.log("Using", deployer.address, "as the deployer");

  const stablePoolConstructors = new StablePoolConstructors(api, deployer);

  const tUSDCtUSDTAddress = await stablePoolConstructors
    .newStable(
      [tUSDC, tUSDT],
      [6, 6],
      200, // A
      deployer.address,
      100000, // trade fee 0.01%
      200000000, // protocol fee 20%
      null // no protocol fee receiver
    )
    .then((res) => res.address);

  const wAZEROsAZEROAddress = await stablePoolConstructors
    .newRated(
      [wAZERO, sAZERO],
      [12, 12],
      [null, rateProvider],
      3600000, // 1h
      200, // A
      deployer.address,
      400000, // trade fee 0.04%
      200000000, // protocol fee 20%
      null // no protocol fee receiver
    )
    .then((res) => res.address);

  const addresses = {
    usdc_usdt_pool: tUSDCtUSDTAddress,
    azero_sazero_pool: wAZEROsAZEROAddress,
  };
  console.log("addresses:", addresses);

  storeAddresses(addresses);

  await api.disconnect();
  console.log("Done");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
