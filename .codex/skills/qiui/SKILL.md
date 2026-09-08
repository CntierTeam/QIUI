---
name: qiui
description: >-
  Develop and operate the independent QIUI (Cellmate) Rust CLI (`qiui`: cloud
  API + BLE, protocol-compatible with app 7.0.51). Covers EncryptUtil AES-CBC,
  discover-api / login / feign calls, GATT profiles (fee7/36f5…), cloud→hex write
  path, and mock BLE smoke. Trigger on: QIUI, Cellmate, qiui CLI, fee7, 36f5,
  EncryptUtil, toyCellmateBluetooth, getToyToken, BLUETOOTH_COMMAND, nokelock.
license: GPL-3.0-only
metadata:
  short-description: Independent QIUI Cellmate API/BLE CLI
---

# QIUI

Independent Rust CLI for QIUI (Cellmate) **cloud API + BLE**. Repo root is the Cargo package `qiui`.
Protocol details live in this skill (not as project `docs/*.md`).

## Hard rules

1. Prefer **boring code**. No speculative abstractions or drive-by refactors.
2. **GATT UUIDs / Nokelock plaintext builders** live only in `src/protocol/`. Change profiles → keep `qiui profiles` output aligned.
3. **API / password / BT-command AES** lives only in `src/crypto.rs`. Keys/IV must stay compatible with QIUI `EncryptUtil` + `libsecret_jni`.
4. **BLE I/O** only through `ble::Session` / `ble::MockSession`. CLI must not call `btleplug` directly.
5. **Login** persists `token` + identity to `~/.config/qiui/config.toml` (`src/config.rs`). Authed commands need that token (or `--token` / `QIUI_TOKEN`).
6. Do **not** reintroduce a repo-root `docs/PROTOCOL.md`; protocol reference stays in this skill.

## Layout

| Path | Role |
|------|------|
| `src/main.rs` | clap subcommands |
| `src/api.rs` | discover, encrypted feign POST, Cellmate BT helpers |
| `src/crypto.rs` | AES-256-CBC keys/IV + encrypt/decrypt helpers |
| `src/protocol/mod.rs` | GATT `Profile` + optional Nokelock/device AES |
| `src/ble/mod.rs` | `Session` (btleplug) + `MockSession` |
| `src/config.rs` | `~/.config/qiui/config.toml` |

## Global flags / config

| Flag / env | Effect |
|------------|--------|
| `--base` / `QIUI_BASE` | API root override |
| `--token` / `QIUI_TOKEN` | auth token override |
| `--verbose` | tracing filter `debug` (else `info`) |

Resolution for base/token: **CLI flag → env → `config.toml` → `DEFAULT_BASE`**.

Config path: `~/.config/qiui/config.toml` fields: `base_url`, `token`, `uid`, `user_id`, `nickname`, `user_name`.

Authed commands (`devices`, `get-toy-token`, `close-lock`, `decry-notify`): bail if no token.

## Common workflows

### Build / run

```bash
cd "$(git rev-parse --show-toplevel)"
# export https_proxy=http://127.0.0.1:7897   # if needed
cargo build --release
cargo run -- discover-api
cargo run -- login -u you@example.com -p 'secret'
cargo run -- write --mock --address AA:BB:CC:DD:EE:FF --hex 06010101deadbeef
```

### Verify (no hardware / optional network)

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -q -- crypto encrypt-api '{"hello":"world"}'
cargo run -q -- crypto decrypt-api "$(cargo run -q -- crypto encrypt-api '{"hello":"world"}')"
cargo run -q -- profiles
cargo run -q -- write --mock --address AA:BB:CC:DD:EE:FF --hex 06010101000000000000000000000000
```

Live API: `discover-api` → `login` → `whoami` / `devices` → `get-toy-token` / `close-lock` → `write` (or `--mock`).

### Protocol / feature work

1. Crypto change → `src/crypto.rs` + unit tests (key lengths / roundtrip).
2. New feign path → `src/api.rs` (`post_encrypted`); keep envelope `{state,message,data,timeStamp}`.
3. New GATT family → `Profile` in `src/protocol/mod.rs`; wire CLI `--profile`.
4. BLE behavior → `src/ble/mod.rs` only; smoke with `--mock`.
5. Re-run `cargo test` and mock write smoke.

Keys, hosts, login fields, GATT table, cloud→hex path: read [references/protocol.md](references/protocol.md).

## CLI usage (agent recipes)

Prefer `cargo run -- <cmd>` from repo root unless `qiui` is installed. Do **not** invent flags; source of truth is `src/main.rs` / `qiui <cmd> --help`.

### `discover-api`

Purpose: fetch & decrypt dynamic API host list.

```bash
cargo run -- discover-api
```

Writes `base_url` to config. Prints `base=`, `http_url=`, `http_image=`, `http_chat=`. Next: `login`.

### `login`

Purpose: password login; persist session.

| Arg | Required | Notes |
|-----|----------|-------|
| `-u` / `--user` | yes | email or phone |
| `-p` / `--password` | yes | plaintext; encrypted with PWD key before POST |
| `--phone` | no | `loginType=3`; default email `loginType=2` |

```bash
cargo run -- login -u you@example.com -p 'secret'
cargo run -- login -u 13800138000 -p 'secret' --phone
cargo run -- --base https://appapi.qiuitoy.com login -u you@example.com -p 'secret'
```

Writes `base_url`, `token`, `uid`, `user_id`, `nickname`, `user_name`. Prints `saved=<path>`. Next: `whoami` / `devices`.

### `whoami`

Purpose: dump saved identity (local only, no network).

```bash
cargo run -- whoami
```

### `devices`

Purpose: list bound toys (`getUserBindingToyDevices`). Needs token.

```bash
cargo run -- devices
cargo run -- --token "$QIUI_TOKEN" devices
```

Pretty JSON → take `toyUid` for token/lock commands.

### `get-toy-token` / `close-lock`

Purpose: cloud → **plaintext BLE hex** (one line stdout).

| Arg | Required |
|-----|----------|
| `--toy-uid` | yes |

```bash
HEX=$(cargo run -q -- get-toy-token --toy-uid '<toyUid>')
HEX=$(cargo run -q -- close-lock --toy-uid '<toyUid>')
cargo run -- write --mock --address AA:BB:CC:DD:EE:FF --hex "$HEX"
```

Next: `write --address … --hex …` (real or `--mock`).

### `decry-notify`

Purpose: cloud decrypt of notify hex (`decryBluetoothCommand`). Needs token.

```bash
cargo run -- decry-notify --hex '<notify_hex>'
```

### `crypto` (local, offline)

| Subcommand | Arg | Notes |
|------------|-----|-------|
| `encrypt-api` | `<TEXT>` | API body → Base64 |
| `decrypt-api` | `<B64>` | API ciphertext |
| `encrypt-pwd` | `<TEXT>` | login password field |
| `encrypt-bt` | `<TEXT>` | BT command key → Base64 |
| `decrypt-bt` | `<B64>` | inverse of encrypt-bt |

```bash
cargo run -q -- crypto encrypt-api '{"hello":"world"}'
cargo run -q -- crypto encrypt-pwd 'secret'
```

### `profiles`

Lists GATT profiles (`name`, service/write/notify). Valid `--profile` values for scan/write.

```bash
cargo run -q -- profiles
```

Default profile name for scan/write: `cellmate`.

### `scan`

| Arg | Default |
|-----|---------|
| `--seconds` | `5` |
| `--profile` | `cellmate` |

Needs BLE adapter. Prints `addr\tname` lines.

```bash
cargo run -- scan --seconds 10 --profile cellmate
```

### `write`

| Arg | Required | Default | Notes |
|-----|----------|---------|-------|
| `--address` | yes | — | MAC |
| `--hex` | yes | — | plaintext hex from cloud |
| `--profile` | no | `cellmate` | must exist in `profiles` |
| `--wait-ms` | no | `3000` | notify wait; `0` = skip |
| `--mock` | no | off | `MockSession`; no adapter |

```bash
cargo run -- write --mock --address AA:BB:CC:DD:EE:FF --hex 06010101000000000000000000000000
cargo run -- write --address AA:BB:CC:DD:EE:FF --hex "$HEX" --wait-ms 5000
```

Real path prints `wrote=` and optional `notify=`; mock prints `notify_mock=`.

## Install this skill into Codex

Canonical copy: repo `.codex/skills/qiui/`.

```bash
ln -sfn "$(git rev-parse --show-toplevel)/.codex/skills/qiui" \
  "${CODEX_HOME:-$HOME/.codex}/skills/qiui"
```

Destination: `${CODEX_HOME:-$HOME/.codex}/skills/qiui`.

## Out of scope (do not expand unless asked)

- Full mobile App UI parity / non-Cellmate product surfaces
- Bundling third-party app source dumps into the crate
- Re-adding project markdown as the protocol source of truth
