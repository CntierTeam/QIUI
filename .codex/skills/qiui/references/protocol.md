# QIUI protocol reference

Protocol reference for the independent `qiui` CLI (compatible with QIUI/Cellmate cloud API + BLE; app protocol version `7.0.51`). Read when changing crypto, feign paths, or GATT profiles.

## API crypto (`EncryptUtil`)

- Algorithm: **AES-256-CBC / PKCS7 (PKCS5Padding)**
- IV: `3*1&f6O%6&Pa5d_8`
- `API_SECRET_STR` (key index `0` ⊕ package `cellmate.qiui.com`):  
  `87dQ_P5F#9&K*G6b9&f%8!d&06@(c_*a`
- `PWD_STR`: `10kB8mdiaWRxFkNQD8Xtb6fm>Cg5N_QX`
- `BLUETOOTH_COMMAND`: `VfttrWZYZ5QVUe>})C5x?oYWVewxVy!]`
- `TIME_STAMP`: `LpyaN2dvT3CbxRNJ>^*kjjAsq=^e*cD5`
- Also in crate (less used by CLI): `BLUETOOTH_ADDRESS`, `MESSAGE` — see `src/crypto.rs`

Request: POST body = Base64(AES(JSON)). Header `token` when logged in.  
Response: Base64 ciphertext → decrypt → JSON envelope `{state,message,data,timeStamp}`.

Success `state` markers treated loosely: `success` / `ok` / `200` / `2000`; fail on `failed` / `error`.

## Hosts

Dynamic list:

`https://qiui-cn.oss-cn-shenzhen.aliyuncs.com/file/urlList/api.html`

Decrypt with `API_SECRET_STR` → e.g. `CH_HTTP_URL="appapi.qiuitoy.com/api"` → base `https://appapi.qiuitoy.com` (strip trailing `/api`).  
Default if undiscovered: `https://appapi.qiuitoy.com` (`DEFAULT_BASE`).

## Login

`POST /feign/userInfo/login`

| Field | Notes |
|-------|-------|
| platformType | `"2"` (Android) |
| userName | email or phone |
| loginType | `"2"` email+pwd · `"3"` phone+pwd |
| passWord | AES with `PWD_STR` (`encrypt_password`) |
| versionNum | `7.0.51` (`APP_VERSION`) |
| deviceType | `"2"` |
| appLanguage | `"0"` |
| phoneId | CLI generates `qiui-cli-<uuid>` |
| blackBox | `""` |

Saves `token`, `uid`, `userId`, `nickname`, `user_name`, `base_url` to `~/.config/qiui/config.toml`.

## Feign paths used by CLI

| Path | Role |
|------|------|
| `/feign/userInfo/login` | password login |
| `/feign/toyUserBinding/getUserBindingToyDevices` | bound toys |
| `/feign/toyCellmateBluetooth/getToyToken` | cloud BLE hex (session token cmd) |
| `/feign/toyCellmateBluetooth/toyCloseLock` | cloud BLE hex (close/lock) |
| `/feign/toyCellmateBluetooth/decryBluetoothCommand` | server decrypt of notify hex |

### CLI mapping (brief)

| CLI | Feign / local |
|-----|---------------|
| `discover-api` | OSS urlList decrypt → `base_url` (no feign) |
| `login` | `/feign/userInfo/login` |
| `devices` | `/feign/toyUserBinding/getUserBindingToyDevices` |
| `get-toy-token` | `/feign/toyCellmateBluetooth/getToyToken` |
| `close-lock` | `/feign/toyCellmateBluetooth/toyCloseLock` |
| `decry-notify` | `/feign/toyCellmateBluetooth/decryBluetoothCommand` |
| `crypto …` | local `src/crypto.rs` only |
| `profiles` / `scan` / `write` | GATT in `src/protocol/` + BLE I/O |

User-facing recipes: repo `README.md` and skill `SKILL.md` (not duplicated here).

Cellmate cloud bodies:

- `getToyToken` / `toyCloseLock`: `toyUid` = AES(`TIME_STAMP`, `"{uid}_{millis}"`)
- `decryBluetoothCommand`: `lockCommand` = AES(`BLUETOOTH_COMMAND`, hex string)
- Response `data` string/object → decrypt with `BLUETOOTH_COMMAND` → **plaintext hex** for GATT write

## BLE write path (cloud → device)

1. Cloud returns field encrypted with `BLUETOOTH_COMMAND`
2. Decrypt → **hex string**
3. Decode hex → bytes
4. Write GATT characteristic (`WriteType::WithResponse`)
5. Optional: wait notify; may upload hex via `decryBluetoothCommand`

## GATT profiles (`src/protocol/mod.rs`)

Short UUIDs expanded as `0000xxxx-0000-1000-8000-00805f9b34fb`.

| Profile CLI name | Service | Write | Notify |
|------------------|---------|-------|--------|
| cellmate | fee7 | 36f5 | 36f6 |
| fee5 | fee5 | 3ff5 | 3ff6 |
| keypod-metal-20 | 0b30 | 0b32 | 0b31 |
| pearflower3 | ac3a | ac3b | ac3c |
| ae3 | ae3a | ae3b | ae3c |
| 8ac0 | 8ac0 | 8ac1 | 8ac2 |
| ffa0 | ffa0 | ffa1 | ffa2 |

Notify CCCD: `00002902-0000-1000-8000-00805f9b34fb`.

Default `--profile` for scan/write: `cellmate`.

## Local Nokelock helpers (optional)

Modern QIUI usually gets ciphertext from cloud. When you have a 16-byte `lockKey`, `protocol::nokelock` builds 16-byte plaintext blocks and `protocol::device_aes` does AES-128-ECB.

| Builder | Opcode lead |
|---------|-------------|
| `get_token_plain` | `06 01 01 01` + random |
| `get_battery_plain` | `02 01 01 01` + 4-byte token |
| `unlock_plain` | `05 01 06` + 6-byte password + token |
| token notify parse | `06 02 …` → 4-byte session token |

## Mock BLE

`MockSession::write_hex` stores last write and returns a zero-filled notify buffer (len ≥ 16). Use for CI without adapter.
