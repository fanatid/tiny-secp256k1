import { test } from "tape";
import { __addon as secp256k1 } from "../";

test("addon exists", (t) => {
  t.notEqual(secp256k1, null);
  t.doesNotThrow(() => {
    secp256k1.__initializeContext();
  });

  t.end();
});
