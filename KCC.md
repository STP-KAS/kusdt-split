# Draft KCC-20 field map

This file maps the lab tag onto the Draft KCC-0020 state order. The order is amount, owner, owner_scheme, borrow_scheme, borrow_guard, extension_commitment.

The tag is a teaching balance in `guest/split.mjs`. The coins sit in `contracts/KusdtLock.sil`. That script has no token fields.

| Draft field | This lab |
| --- | --- |
| amount | The KUSDT cents on the account. A move keeps the sum. |
| owner | The account name that holds those cents. |
| owner_scheme | Unset. This lab does not choose a KCC-2 digest. |
| borrow_scheme | Unset. There is no borrow. |
| borrow_guard | Unset. |
| extension_commitment | Unset on the covenant. The freeze rule lives in the guest program, outside this draft. |

`contracts/KusdtLock.sil` locks sompi and pays that same amount back to the holder key. It does not store this table.

The SilverScript example `kcc20.sil` in the compiler tree is a different four-field program. This map does not use it.
