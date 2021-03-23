import { randomBytes } from "crypto";
import * as secp256k1 from "../../lib/index.js";

const eq = (v1, v2) =>
  v1.length === v2.length && v1.every((v, i) => v === v2[i]);
const toHex = (v) =>
  v instanceof Uint8Array ? Buffer.from(v).toString("hex") : v;

function isValidData(data) {
  return (
    secp256k1.isPoint(data.pubkey) &&
    secp256k1.isPoint(data.pubkey_uncompressed) &&
    secp256k1.isPoint(data.pubkey2) &&
    secp256k1.isPointCompressed(data.pubkey) &&
    secp256k1.isPointCompressed(data.pubkey2) &&
    secp256k1.isPrivate(data.seckey) &&
    secp256k1.isPrivate(data.seckey2) &&
    secp256k1.pointAdd(data.pubkey, data.pubkey2) !== null &&
    secp256k1.pointAddScalar(data.pubkey, data.tweak) !== null &&
    secp256k1.pointAddScalar(data.pubkey2, data.tweak) !== null &&
    eq(secp256k1.pointCompress(data.pubkey, false), data.pubkey_uncompressed) &&
    eq(secp256k1.pointFromScalar(data.seckey, true), data.pubkey) &&
    eq(
      secp256k1.pointFromScalar(data.seckey, false),
      data.pubkey_uncompressed
    ) &&
    eq(secp256k1.pointFromScalar(data.seckey2, true), data.pubkey2) &&
    secp256k1.pointMultiply(data.pubkey, data.tweak) !== null &&
    secp256k1.pointMultiply(data.pubkey2, data.tweak) !== null &&
    secp256k1.privateAdd(data.seckey, data.tweak) !== null &&
    secp256k1.privateAdd(data.seckey2, data.tweak) !== null &&
    secp256k1.privateSub(data.seckey, data.tweak) !== null &&
    secp256k1.privateSub(data.seckey2, data.tweak) !== null &&
    secp256k1.verify(
      data.hash,
      data.pubkey,
      secp256k1.sign(data.hash, data.seckey)
    )
  );
}

export function generate() {
  for (;;) {
    const seckey = new Uint8Array(randomBytes(32));
    const seckey2 = new Uint8Array(randomBytes(32));

    const data = {
      seckey,
      pubkey: secp256k1.pointFromScalar(seckey, true),
      pubkey_uncompressed: secp256k1.pointFromScalar(seckey, false),
      seckey2,
      pubkey2: secp256k1.pointFromScalar(seckey2, true),
      tweak: new Uint8Array(randomBytes(32)),
      hash: new Uint8Array(randomBytes(32)),
      entropy: new Uint8Array(randomBytes(32)),
    };

    if (isValidData(data)) return data;
  }
}

const lineDash = new Array(80).fill("-").join("");
const lineEq = new Array(80).fill("=").join("");
function print(items) {
  let firstPrint = true;
  for (const item of items) {
    if (firstPrint) {
      console.log(lineEq);
      firstPrint = false;
    }
    console.log(`Method: ${item.name}`);
    console.log(lineDash);
    for (let i = 0; i < item.args.length; ++i) {
      console.log(`Arg${(i + 1).toString()}: ${toHex(item.args[i])}`);
    }
    console.log(`Result: ${toHex(secp256k1[item.name](...item.args))}`);
    console.log(lineEq);
  }
}

const data = generate();
print([
  { name: "isPoint", args: [data.pubkey_uncompressed] },
  { name: "isPointCompressed", args: [data.pubkey] },
  { name: "isPrivate", args: [data.seckey] },
  { name: "pointAdd", args: [data.pubkey, data.pubkey2] },
  { name: "pointAddScalar", args: [data.pubkey, data.tweak] },
  { name: "pointCompress", args: [data.pubkey_uncompressed, true] },
  { name: "pointFromScalar", args: [data.seckey] },
  { name: "pointMultiply", args: [data.pubkey, data.tweak] },
  { name: "privateAdd", args: [data.seckey, data.tweak] },
  { name: "privateSub", args: [data.seckey, data.tweak] },
  { name: "sign", args: [data.hash, data.seckey] },
  { name: "signWithEntropy", args: [data.hash, data.seckey, data.entropy] },
  {
    name: "verify",
    args: [data.hash, data.pubkey, secp256k1.sign(data.hash, data.seckey)],
  },
]);
