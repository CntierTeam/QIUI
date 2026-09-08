# qiui

独立的 QIUI（Cellmate）控制 CLI（Rust）：兼容云端 API 与 BLE 设备，协议版本对齐 `7.0.51`。

仓库：[CntierTeam/QIUI](https://github.com/CntierTeam/QIUI)

协议细节不在本仓库文档中；见 Codex skill **`$qiui`**（仓库内 `.codex/skills/qiui/`，含 `references/protocol.md`）。

## 一键安装（从 GitHub Release）

```bash
curl -fsSL https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.sh | bash
```

默认安装：

- 二进制 → `~/.local/bin/qiui`
- Codex skill → `~/.codex/skills/qiui`

Windows（PowerShell）：

```powershell
irm https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.ps1 | iex
# 或指定版本：
.\scripts\install.ps1 -Version v0.1.1 -Force
```

默认安装到 `%LOCALAPPDATA%\qiui\bin\qiui.exe`（脚本会提示把该目录加入 PATH）。

常用选项：

```bash
# 指定版本
curl -fsSL https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.sh \
  | bash -s -- --version v0.1.1 --force

# 只装二进制 / 只装 skill
bash scripts/install.sh --bin-only
bash scripts/install.sh --skill-only --force

# 开发机：从本地源码安装
./scripts/install.sh --from-source --symlink-skill --force

# 卸载
./scripts/install.sh --uninstall
```

装好后确保 `~/.local/bin` 在 `PATH` 里，然后：`qiui profiles`。

## 构建 / 运行

```bash
# 如需代理：export https_proxy=http://127.0.0.1:7897
cargo build --release
cargo run -- <子命令>
# 或：
# cargo install --path .
# qiui <子命令>
```

下文示例以已安装的 `qiui` 为准；开发时把 `qiui` 换成 `cargo run --` 即可。

## 配置与全局参数

| 来源 | 说明 |
|------|------|
| `~/.config/qiui/config.toml` | `discover-api` / `login` 写入的 `base_url`、`token`、`uid`、`userId`、`nickname`、`user_name` |
| 环境变量 `QIUI_BASE` / `QIUI_TOKEN` | 覆盖 base / token |
| 全局参数 `--base` / `--token` / `--verbose` | 同上；命令行优先于环境变量与 config |

解析优先级（base / token）：**`--base`/`--token` → `QIUI_BASE`/`QIUI_TOKEN` → config.toml → 内置默认 base**。

`--verbose`：把日志级别提到 `debug`（默认 `info`）。

需登录态的命令：`devices`、`get-toy-token`、`close-lock`、`decry-notify`。缺 token 时会提示先 `login` 或传 `--token` / `QIUI_TOKEN`。

典型流程：`discover-api` → `login` → `whoami` / `devices` → `get-toy-token` / `close-lock` → `write`。

---

## 命令用法

### `discover-api` — 解析动态 API 主机

拉取并解密官方 urlList，得到可用 API 根地址。

```bash
qiui discover-api
# 可选覆盖（一般不需要）：
qiui --verbose discover-api
```

**输出示例字段**：`base=`、`http_url=`、`http_image=`、`http_chat=`。

**写入 config**：`base_url`。

**下一步**：`login`。

---

### `login` — 密码登录并保存会话

| 参数 | 必填 | 说明 |
|------|------|------|
| `-u` / `--user` | 是 | 邮箱或手机号 |
| `-p` / `--password` | 是 | 明文密码（本地用 PWD 密钥加密后提交） |
| `--phone` | 否 | 有则 `loginType=3`（手机）；默认 `loginType=2`（邮箱） |

```bash
# 邮箱登录
qiui login -u you@example.com -p 'secret'

# 手机登录
qiui login -u 13800138000 -p 'secret' --phone

# 已有 base 时可跳过 discover；也可显式指定
qiui --base https://appapi.qiuitoy.com login -u you@example.com -p 'secret'
```

**输出**：`state=ok`、`token`、`uid`、`userId`、`nickname`、`saved=<config 路径>`。

**写入 config**：`base_url`、`token`、`uid`、`user_id`、`nickname`、`user_name`。

**下一步**：`whoami` 或 `devices`。

---

### `whoami` — 查看本地已保存身份

只读 `config.toml`，不访问网络。

```bash
qiui whoami
```

**输出**：`config=` 路径，以及 `base` / `token` / `uid` / `userId` / `nickname` / `user`。

**下一步**：确认有 token 后跑 `devices`。

---

### `devices` — 列出绑定设备

调用 `getUserBindingToyDevices`，需要登录态。

```bash
qiui devices
qiui --token '<token>' devices   # 临时覆盖 token
```

**输出**：绑定设备 JSON（pretty）。从中取玩具的 `toyUid`（或等价字段）供后续命令使用。

**下一步**：`get-toy-token --toy-uid …` 或 `close-lock --toy-uid …`。

---

### `get-toy-token` — 云端下发会话类 BLE 明文 hex

| 参数 | 必填 | 说明 |
|------|------|------|
| `--toy-uid` | 是 | 绑定列表中的玩具 UID |

```bash
qiui get-toy-token --toy-uid '<toyUid>'
HEX=$(qiui get-toy-token --toy-uid '<toyUid>')
```

**输出**：一行明文 hex（可直接给 `write --hex`）。

**下一步**：`write --address <MAC> --hex "$HEX"`（或先 `--mock` 冒烟）。

---

### `close-lock` — 云端下发关锁/上锁类 BLE 明文 hex

| 参数 | 必填 | 说明 |
|------|------|------|
| `--toy-uid` | 是 | 同上 |

```bash
qiui close-lock --toy-uid '<toyUid>'
```

**输出**：一行明文 hex。

**下一步**：同 `get-toy-token`，用 `write` 写入设备。

---

### `decry-notify` — 云端解密设备 notify hex

| 参数 | 必填 | 说明 |
|------|------|------|
| `--hex` | 是 | 设备 notify 上报的 hex 字符串 |

```bash
qiui decry-notify --hex '<notify_hex>'
```

**输出**：云端解密结果 JSON（pretty）。常接在真实 `write` 收到 `notify=` 之后。

---

### `crypto` — 本地加解密助手（不访问网络）

子命令均为位置参数，结果打印到 stdout：

| 子命令 | 参数 | 用途 |
|--------|------|------|
| `encrypt-api <TEXT>` | 明文 JSON/文本 | API 请求体加密 → Base64 |
| `decrypt-api <B64>` | Base64 密文 | API 响应解密 |
| `encrypt-pwd <TEXT>` | 明文密码 | 登录密码字段加密 |
| `encrypt-bt <TEXT>` | 明文 | BLUETOOTH_COMMAND 密钥加密 → Base64 |
| `decrypt-bt <B64>` | Base64 | 对称解密 |

```bash
qiui crypto encrypt-api '{"hello":"world"}'
qiui crypto decrypt-api '<b64>'
qiui crypto encrypt-pwd 'secret'
qiui crypto encrypt-bt '06010101...'
qiui crypto decrypt-bt '<b64>'

# 本地往返自检
qiui crypto decrypt-api "$(qiui crypto encrypt-api '{"hello":"world"}')"
```

---

### `profiles` — 列出内置 GATT 配置

```bash
qiui profiles
```

**输出**：每行 `名称`、`service=`、`write=`、`notify=`。`scan` / `write` 的 `--profile` 必须是其中之一（默认 `cellmate`）。

---

### `scan` — BLE 扫描

| 参数 | 默认 | 说明 |
|------|------|------|
| `--seconds` | `5` | 扫描时长（秒） |
| `--profile` | `cellmate` | GATT 配置名（见 `profiles`） |

需要本机蓝牙适配器；不写 config。

```bash
qiui scan
qiui scan --seconds 10 --profile cellmate
qiui scan --profile fee5
```

**输出**：每行 `地址` + TAB + 设备名（可能为空）。记下 MAC 给 `write --address`。

---

### `write` — 连接设备并写入 hex

| 参数 | 必填 | 默认 | 说明 |
|------|------|------|------|
| `--address` | 是 | — | 蓝牙地址（如 `AA:BB:CC:DD:EE:FF`） |
| `--hex` | 是 | — | 明文 hex（通常来自 `get-toy-token` / `close-lock`） |
| `--profile` | 否 | `cellmate` | GATT 配置 |
| `--wait-ms` | 否 | `3000` | 写后等待 notify 的毫秒；`0` 表示不等待 |
| `--mock` | 否 | 关 | 无硬件冒烟：不连真实 BLE |

```bash
# 无硬件冒烟（CI / 本地自检）
qiui write --mock --address AA:BB:CC:DD:EE:FF --hex 06010101000000000000000000000000

# 真实设备：云端 hex → 写入
HEX=$(qiui get-toy-token --toy-uid '<toyUid>')
qiui write --address AA:BB:CC:DD:EE:FF --hex "$HEX"
qiui write --address AA:BB:CC:DD:EE:FF --hex "$HEX" --profile cellmate --wait-ms 5000

# 写完立刻用云端解密 notify（若有 notify= 输出）
qiui decry-notify --hex '<notify_hex>'
```

**输出**：`wrote=<hex>`；真实模式另有 `notify=<hex>`（超时则打印到 stderr）；`--mock` 时为 `notify_mock=`。

---

## 端到端配方（速查）

```bash
# 1) 解析 API + 登录
qiui discover-api
qiui login -u you@example.com -p 'secret'
qiui whoami

# 2) 取设备 → 取云端 BLE hex → 写入（或 mock）
qiui devices
HEX=$(qiui get-toy-token --toy-uid '<toyUid>')
qiui write --mock --address AA:BB:CC:DD:EE:FF --hex "$HEX"
# 有硬件时：
# qiui scan --seconds 8
# qiui write --address <MAC> --hex "$HEX"

# 3) 关锁类命令同理
# qiui close-lock --toy-uid '<toyUid>'
```

环境覆盖示例：

```bash
export QIUI_BASE=https://appapi.qiuitoy.com
export QIUI_TOKEN='...'
qiui devices
# 或一次性：
qiui --base https://appapi.qiuitoy.com --token '...' --verbose devices
```

## CI / Release

| Workflow | 触发 | 作用 |
|----------|------|------|
| [CI](.github/workflows/ci.yml) | `push`/`PR` → `main` | `cargo test`、clippy、离线 CLI 冒烟 |
| [Release](.github/workflows/release.yml) | 推送 tag `v*.*.*` | 多平台 release 构建并发布 GitHub Release |

Release 产物：

- `qiui-<target>.tar.gz` — 预编译二进制（linux x86_64/aarch64、macOS aarch64）
- `qiui-x86_64-pc-windows-msvc.zip` — Windows x86_64（含 `qiui.exe`）
- `qiui-skill.tar.gz` — Codex skill
- `install.sh` / `install.ps1` — 安装脚本副本
- 对应 `.sha256`

打 tag 发版：

```bash
git tag v0.1.1
git push origin v0.1.1
```

## 协议与开发

- Codex skill：`$qiui` → `.codex/skills/qiui/SKILL.md`
- 密钥、主机、GATT、云→hex 路径：`.codex/skills/qiui/references/protocol.md`
- 请勿在仓库再维护长版 `docs/PROTOCOL.md`

## License

GPL-3.0-only — see [LICENSE](LICENSE)。
