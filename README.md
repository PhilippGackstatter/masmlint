# MASM Linter

A linter for [Miden Assembly](https://0xmiden.github.io/miden-docs/imported/miden-vm/src/user_docs/assembly/main.html).

## Installation

Install via cargo:

```sh
cargo install masmlint
```

## Usage



## Lints

### `PushImmediate`

**What it does**

Checks if _immediate_ values are pushed before instructions that could take that immediate directly.

**Why is this bad?**

It is faster to read and understand `lt.2` than `push.2 lt`.

**Example**

```
Error:   x operand is pushed before an instruction that can take an immediate value
     ,-[lib/tx.masm:227:9]
 226 |     dup neq.0 assert.err=ERR_TX_INVALID_EXPIRATION_DELTA
 227 |     dup push.EXPIRY_UPPER_LIMIT lt assert.err=ERR_TX_INVALID_EXPIRATION_DELTA
     :         ^^^^^^^^^^^^^|^^^^^^^^^^^^
     :                      `-- instruction can be rewritten to take the immediate directly
 228 |     # => [block_height_delta]
     `----
  help: use the instruction in its immediate form `lt.EXPIRY_UPPER_LIMIT`
```

### `BareAssert`

**What it does**

Checks for assertions without error messages.

**Why is this bad?**

Without error messages, failed assertions will produce a message like "assertion failed at clock cycle 123 with error code: 0". This is an unhelpful start to finding the cause of that error.

**Example**

```
Error:   x assert without error message
     ,-[lib/prologue.masm:146:18]
 145 |     # assert that sequential hash matches the precomputed kernel commitment
 146 |     movup.4 drop assert_eqw
     :                  ^^^^^|^^^^
     :                       `-- does not include an error message
 147 |     # OS => [kernel_version]
     `----
  help: use the instruction with a helpful error message, e.g.
        `assert_eqw.err=helpful error message`
```