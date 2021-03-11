import * as fs from "fs";
import { URL } from "url";

export function loadJSON(location) {
  const bytes = fs.readFileSync(new URL(location, import.meta.url));
  return JSON.parse(bytes);
}

export function fromHex(data) {
  return new Uint8Array(Buffer.from(data, "hex"));
}

export function toHex(data) {
  return Buffer.from(data).toString("hex");
}
