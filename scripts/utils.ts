import fs from "fs";

import { Keyring } from "@polkadot/api";

export const RPC_URL = "wss://ws.test.azero.dev";
export const DEPLOYER_SEED = "section token crazy better fitness reflect similar witness friend east monitor police";

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