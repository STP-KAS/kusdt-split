// One declared batch. The first accepted step lands. Later steps are Retry.
// A gas step is refused and does not take the landing.
// Freeze blocks a KUSDT move. A POCencept move still lands.
// This program does not hold sompi and does not spend a covenant.

export const NEVER_GAS = "NEVER_GAS";

export function open(accounts) {
  const next = {};
  for (const [name, row] of Object.entries(accounts)) {
    next[name] = {
      kusdt: BigInt(row.kusdt),
      poc: BigInt(row.poc),
      frozen: Boolean(row.frozen),
    };
  }
  return { accounts: next };
}

function copy(state) {
  const accounts = {};
  for (const [name, row] of Object.entries(state.accounts)) {
    accounts[name] = { kusdt: row.kusdt, poc: row.poc, frozen: row.frozen };
  }
  return { accounts };
}

function row(state, name) {
  const found = state.accounts[name];
  if (!found) {
    throw new Error(`Empty: ${name}`);
  }
  return found;
}

function move(table, from, to, cents, field) {
  const amount = BigInt(cents);
  if (amount <= 0n) {
    return "Empty";
  }
  if (table[from][field] < amount) {
    return "Short";
  }
  table[from][field] -= amount;
  table[to][field] += amount;
  return null;
}

function reject(state, attempt) {
  if (attempt.kind === "freeze") {
    row(state, attempt.address);
    return null;
  }
  if (attempt.kind === "kusdt") {
    const from = row(state, attempt.from);
    row(state, attempt.to);
    if (from.frozen) {
      return "Frozen";
    }
    if (from.kusdt < BigInt(attempt.cents) || BigInt(attempt.cents) <= 0n) {
      return from.kusdt < BigInt(attempt.cents) ? "Short" : "Empty";
    }
    return null;
  }
  if (attempt.kind === "poc") {
    const from = row(state, attempt.from);
    row(state, attempt.to);
    if (from.poc < BigInt(attempt.cents) || BigInt(attempt.cents) <= 0n) {
      return from.poc < BigInt(attempt.cents) ? "Short" : "Empty";
    }
    return null;
  }
  return "Empty";
}

function apply(state, attempt) {
  if (attempt.kind === "freeze") {
    row(state, attempt.address).frozen = true;
    return;
  }
  if (attempt.kind === "kusdt") {
    const reason = move(state.accounts, attempt.from, attempt.to, attempt.cents, "kusdt");
    if (reason) {
      throw new Error(reason);
    }
    return;
  }
  if (attempt.kind === "poc") {
    const reason = move(state.accounts, attempt.from, attempt.to, attempt.cents, "poc");
    if (reason) {
      throw new Error(reason);
    }
  }
}

// attempts stay in the order they were declared.
// gas is NEVER_GAS and does not occupy the one landing.
export function landBatch(state, attempts) {
  const next = copy(state);
  let landed = null;
  const refused = [];
  for (const attempt of attempts) {
    if (attempt.kind === "gas") {
      refused.push({ ...attempt, reason: NEVER_GAS });
      continue;
    }
    if (landed) {
      refused.push({ ...attempt, reason: "Retry" });
      continue;
    }
    let reason;
    try {
      reason = reject(next, attempt);
    } catch (error) {
      reason = error.message.startsWith("Empty") ? "Empty" : error.message;
    }
    if (reason) {
      refused.push({ ...attempt, reason });
      continue;
    }
    apply(next, attempt);
    landed = { ...attempt };
  }
  return { state: next, landed, refused };
}
