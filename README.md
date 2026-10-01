> Experimental only. Not a product. Not money. Not Kaspa Core.
>
> Do not use wallet integrations on this GitHub. STP remains a clown.
> [DISCLAIMER.md](DISCLAIMER.md)

# KUSDT split

Experimental lab. Testnet teaching numbers. Not a dollar. Not Kaspa core. Not a till. The spend gate stays shut.

Two programs:

- `contracts/KusdtLock.sil` is a SilverScript contract. It locks tKAS and pays the full amount back to the holder. The fee is another input. The script has one entry, `redeem`.
- `guest/split.mjs` is a local program. One declared step lands. A freeze blocks a KUSDT move. A POCencept move still lands. Using the tag as gas is refused. This program does not hold sompi.

`KCC.md` places the tag fields in Draft KCC-0020 order. The freeze is outside that draft.

BitCoffee KUSD is a different object. Maxim's tic-tac-toe guest is a different program. This folder does not submit a transaction and does not read a key.

## Run

Compile the contract with SilverScript v1.0.0 (`pragma silverscript ^0.1.0`; the artifact stamps `compiler_version` 0.1.0):

```text
silverc contracts/KusdtLock.sil --constructor-args contracts/compile-holder.json -o contracts/KusdtLock.json
```

`contracts/compile-holder.json` is 32 fixture bytes so the compiler can bake a holder. It is not a key. The engine test bakes its own throwaway key and does not read that file.

Guest:

```text
node --test guest/split.test.mjs
```

Script engine, from this folder:

```text
cargo test
```

The engine test signs a throwaway key in memory, runs `redeem` against the Kaspa script engine, and checks three failures: a skimmed output, a different recipient, and a different key.

`Cargo.toml` points `silverscript-lang` and `silverscript-abi` at a local checkout of [kaspanet/silverscript](https://github.com/kaspanet/silverscript) tag v1.0.0, commit `3ed9733`. A clone of this repo does not contain that checkout. The script-engine crates are rusty-kaspa rev `a41a333b08848f41bf737b72592e463a6011b8ac`, the same pin silverscript uses. The guest test needs only Node.

On 1 Oct 2026 this desk ran the guest test (5 passed) and `cargo test --test lock` (6 passed). No transaction was submitted. There is no Testnet 10 transaction id.

The 1984 square does not attach this lock to a shop buy.

Intentions are good; thought process is questionable. STP remains delusional. Si vis pacem, para bellum.

Intern at https://sixpack.wtf/
X: https://x.com/StppStp · GitHub: https://github.com/STP-KAS
