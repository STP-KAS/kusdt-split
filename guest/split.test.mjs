import assert from "node:assert/strict";
import test from "node:test";
import { NEVER_GAS, landBatch, open } from "./split.mjs";

const ALICE = "alice";
const BOB = "bob";

function book() {
  return open({
    [ALICE]: { kusdt: 2000n, poc: 2000n, frozen: false },
    [BOB]: { kusdt: 0n, poc: 0n, frozen: false },
  });
}

test("one declared step lands and the second is Retry", () => {
  const batch = landBatch(book(), [
    { kind: "kusdt", from: ALICE, to: BOB, cents: 100n },
    { kind: "poc", from: ALICE, to: BOB, cents: 100n },
  ]);
  assert.equal(batch.landed.kind, "kusdt");
  assert.equal(batch.refused[0].reason, "Retry");
  assert.equal(batch.state.accounts[ALICE].kusdt, 1900n);
  assert.equal(batch.state.accounts[BOB].kusdt, 100n);
  assert.equal(batch.state.accounts[ALICE].poc, 2000n);
  assert.equal(batch.state.accounts[BOB].poc, 0n);
});

test("frozen KUSDT blocks the tag and POCencept still moves", () => {
  const frozen = landBatch(book(), [{ kind: "freeze", address: ALICE }]);
  assert.equal(frozen.landed.kind, "freeze");
  assert.equal(frozen.state.accounts[ALICE].frozen, true);
  assert.equal(frozen.state.accounts[ALICE].kusdt, 2000n);

  const pay = landBatch(frozen.state, [
    { kind: "kusdt", from: ALICE, to: BOB, cents: 100n },
    { kind: "poc", from: ALICE, to: BOB, cents: 100n },
  ]);
  assert.equal(pay.refused[0].reason, "Frozen");
  assert.equal(pay.landed.kind, "poc");
  assert.equal(pay.state.accounts[ALICE].kusdt, 2000n);
  assert.equal(pay.state.accounts[ALICE].poc, 1900n);
  assert.equal(pay.state.accounts[BOB].poc, 100n);

  const poc = landBatch(frozen.state, [
    { kind: "poc", from: ALICE, to: BOB, cents: 100n },
  ]);
  assert.equal(poc.landed.kind, "poc");
  assert.equal(poc.state.accounts[ALICE].poc, 1900n);
  assert.equal(poc.state.accounts[BOB].poc, 100n);
  assert.equal(poc.state.accounts[ALICE].kusdt, 2000n);
  assert.equal(poc.state.accounts[ALICE].frozen, true);
});

test("the tag is refused as gas and the refusal does not take the landing", () => {
  const batch = landBatch(book(), [
    { kind: "gas" },
    { kind: "poc", from: ALICE, to: BOB, cents: 50n },
  ]);
  assert.equal(batch.refused[0].reason, NEVER_GAS);
  assert.equal(batch.landed.kind, "poc");
  assert.equal(batch.state.accounts[BOB].poc, 50n);
});

test("a short KUSDT move does not write", () => {
  const batch = landBatch(book(), [
    { kind: "kusdt", from: ALICE, to: BOB, cents: 9000n },
  ]);
  assert.equal(batch.landed, null);
  assert.equal(batch.refused[0].reason, "Short");
  assert.equal(batch.state.accounts[ALICE].kusdt, 2000n);
});

test("landBatch leaves its input alone", () => {
  const before = book();
  const snapshot = JSON.stringify(before, (_, value) => (typeof value === "bigint" ? value.toString() : value));
  landBatch(before, [{ kind: "kusdt", from: ALICE, to: BOB, cents: 100n }]);
  const after = JSON.stringify(before, (_, value) => (typeof value === "bigint" ? value.toString() : value));
  assert.equal(after, snapshot);
});
