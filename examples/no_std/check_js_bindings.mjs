/**
 * Basic JavaScript bindings check.
 *
 * To run, use the following:
 *   node check_js_bindings.mjs
 */

import assert from "node:assert/strict";
import { deserialize, serialize } from "./js_no_std_bindings/index.js";

console.log("Checking JS bindings");

const value = {
    packet: {
        tag: "A1",
        value: {
            name: "foo",
            version: 42,
            payload: [0xde, 0xad, 0xc0, 0xde],
        },
    },
};

const result = deserialize("Protocol", serialize("Protocol", value));

assert.deepEqual(result.value, value);
assert.equal(result.bytes.length, 0);

console.log("Done!");
