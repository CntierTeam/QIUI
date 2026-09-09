---
name: qiui
description: >-
  Use the independent QIUI (Cellmate) CLI (`qiui`) for cloud login, bound toys,
  cloud→BLE hex commands, and GATT write/scan. Prefer installed `qiui` over
  rebuilding. Trigger on: QIUI, Cellmate, qiui CLI, get-toy-token, close-lock,
  discover-api, write --mock, fee7, 36f5, toyUid, QIUI_TOKEN.
license: GPL-3.0-only
metadata:
  short-description: Use QIUI/Cellmate CLI (login, BLE write)
---

# QIUI — 使用指南

`qiui` 是独立的 QIUI（Cellmate）**云端 API + BLE** 控制 CLI（协议对齐 app `7.0.51`）。

本 skill 教 **怎么用工具**，不是怎么改源码。需要协议细节时再看 [references/protocol.md](references/protocol.md)。

## 使用原则

1. 优先用 **已安装的 `qiui`**（`PATH` 上的二进制）。不要默认去 `cargo build` / 改仓库。
2. **不要编造参数**；不确定就跑 `qiui --help` / `qiui <cmd> --help`。
3. 需要登录态的命令：`devices`、`get-toy-token`、`close-lock`、`decry-notify`。缺 token 就先 `login`，或传 `--token` / `QIUI_TOKEN`。
4. 真机 BLE 写之前，先用云端拿到 **明文 hex**（`get-toy-token` / `close-lock`），再 `write`。无硬件时用 `--mock` 冒烟。
5. 对用户回复用对方语言；命令与路径保持英文原样。

## 安装

仓库：https://github.com/CntierTeam/QIUI

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.sh | bash
```

```powershell
# Windows
irm https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.ps1 | iex
```

默认：

| 平台 | 二进制 | Codex skill（可选） |
|------|--------|---------------------|
| Unix | `~/.local/bin/qiui` | `~/.codex/skills/qiui` |
| Windows | `%LOCALAPPDATA%\qiui\bin\qiui.exe` | `%USERPROFILE%\.codex\skills\qiui` |

确保对应 `bin` 目录在 `PATH` 里，然后：`qiui profiles`。

指定版本 / 强制覆盖：`bash … --version v0.1.1 --force`；PowerShell：`-Version v0.1.1 -Force`。

## 配置

| 来源 | 说明 |
|------|------|
| `~/.config/qiui/config.toml` | `discover-api` / `login` 写入的 `base_url`、`token`、`uid`、`user_id`、`nickname`、`user_name` |
| `QIUI_BASE` / `QIUI_TOKEN` | 覆盖 base / token |
| `--base` / `--token` / `--verbose` | 命令行优先 |

解析顺序：**CLI → 环境变量 → config → 内置默认 base**。

## 典型流程

```bash
qiui discover-api
qiui login -u you@example.com -p 'secret'          # 手机号加 --phone
qiui whoami
qiui devices                                       # 记下 toyUid
HEX=$(qiui get-toy-token --toy-uid '<toyUid>')
qiui scan --seconds 8                              # 真机：找 MAC
qiui write --address <MAC> --hex "$HEX"            # 或先 --mock 验证
```

关锁类：`HEX=$(qiui close-lock --toy-uid '<toyUid>')`，同样 `write`。

## 命令速查

### 全局

```bash
qiui --help
qiui --verbose <cmd>
qiui --base https://appapi.qiuitoy.com --token '…' devices
```

### `discover-api`

拉取并解密动态 API 主机，写入 `base_url`。

```bash
qiui discover-api
```

### `login`

| 参数 | 必填 | 说明 |
|------|------|------|
| `-u` / `--user` | 是 | 邮箱或手机号 |
| `-p` / `--password` | 是 | 明文密码 |
| `--phone` | 否 | 手机登录（`loginType=3`）；默认邮箱 |

```bash
qiui login -u you@example.com -p 'secret'
qiui login -u 13800138000 -p 'secret' --phone
```

成功会打印 `token` / `saved=<config 路径>`。

### `whoami`

只读本地 config，不联网。

```bash
qiui whoami
```

### `devices`

列出绑定设备（需 token）。从 JSON 里取 `toyUid`。

```bash
qiui devices
```

### `get-toy-token` / `close-lock`

向云端要 **一行明文 BLE hex**（stdout），供 `write` 使用。

```bash
HEX=$(qiui get-toy-token --toy-uid '<toyUid>')
HEX=$(qiui close-lock --toy-uid '<toyUid>')
```

### `decry-notify`

把设备 notify 的 hex 交给云端解密。

```bash
qiui decry-notify --hex '<notify_hex>'
```

### `crypto`（本地离线）

调试用：加解密 API / 密码 / BT 载荷。日常控锁一般不需要。

```bash
qiui crypto encrypt-api '{"hello":"world"}'
qiui crypto encrypt-pwd 'secret'
```

### `profiles`

列出可用 GATT profile 名（`scan` / `write` 的 `--profile`）。默认 `cellmate`。

```bash
qiui profiles
```

### `scan`

| 参数 | 默认 |
|------|------|
| `--seconds` | `5` |
| `--profile` | `cellmate` |

需要本机 BLE 适配器。输出 `addr\tname`。

```bash
qiui scan --seconds 10
```

### `write`

| 参数 | 必填 | 默认 | 说明 |
|------|------|------|------|
| `--address` | 是 | — | 设备 MAC |
| `--hex` | 是 | — | 云端返回的明文 hex |
| `--profile` | 否 | `cellmate` | 见 `profiles` |
| `--wait-ms` | 否 | `3000` | 等 notify；`0` 跳过 |
| `--mock` | 否 | 关 | 无硬件冒烟 |

```bash
qiui write --mock --address AA:BB:CC:DD:EE:FF --hex 06010101000000000000000000000000
qiui write --address AA:BB:CC:DD:EE:FF --hex "$HEX" --wait-ms 5000
```

## 常见问题

| 现象 | 处理 |
|------|------|
| 提示缺 token | `qiui login …` 或设 `QIUI_TOKEN` / `--token` |
| API 主机不对 | 先 `qiui discover-api`，或 `--base https://appapi.qiuitoy.com` |
| 无 BLE 适配器 | 用 `write --mock`；真机需系统蓝牙权限（Linux 常需 BlueZ） |
| Windows 装好找不到命令 | 把 `%LOCALAPPDATA%\qiui\bin` 加进用户 PATH 后重开终端 |
| 只要二进制、不要 skill | `install.sh --bin-only` / `install.ps1 -BinOnly` |

## 参考

- 用户文档：仓库 [README.md](https://github.com/CntierTeam/QIUI/blob/main/README.md)
- 协议背景（密钥 / feign / GATT）：[references/protocol.md](references/protocol.md)
- Release：https://github.com/CntierTeam/QIUI/releases
