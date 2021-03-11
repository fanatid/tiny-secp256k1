import test_ecdsa from "./ecdsa.js";
import test_points from "./points.js";
import test_privates from "./privates.js";

import * as wasm from "../lib/index.js";

test_ecdsa(wasm);
test_points(wasm);
test_privates(wasm);
