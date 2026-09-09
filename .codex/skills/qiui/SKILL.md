---
name: qiui
description: >-
  Product usage for the QIUI (Cellmate) CLI `qiui`: install, login, list toys,
  cloud BLE commands, scan/write. Trigger on: QIUI, Cellmate, qiui, 登录,
  get-toy-token, close-lock, devices, write, scan.
license: GPL-3.0-only
metadata:
  short-description: QIUI CLI 产品用法
---

# QIUI

产品：**`qiui`** — QIUI（Cellmate）云端账号 + 蓝牙设备控制命令行。

本 skill **只说明怎么用 CLI**。不要讲源码、协议实现或开发流程。

## 安装

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.sh | bash
```

```powershell
# Windows
irm https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.ps1 | iex
```

装好后确认：

```bash
qiui --help
qiui profiles
```

Unix 默认：`~/.local/bin/qiui`（需在 `PATH`）。  
Windows 默认：`%LOCALAPPDATA%\qiui\bin\qiui.exe`。

指定版本：`--version v0.1.2` / `-Version v0.1.2`；覆盖已有安装加 `--force` / `-Force`。

## 登录与配置

会话保存在 `~/.config/qiui/config.toml`（可用环境变量 `QIUI_BASE`、`QIUI_TOKEN` 或全局参数 `--base`、`--token` 覆盖）。

```bash
qiui discover-api
qiui login -u you@example.com -p '你的密码'
qiui login -u 13800138000 -p '你的密码' --phone   # 手机号
qiui whoami
```

缺登录态时，先 `login`，再跑需要账号的命令。

## 日常用法

```bash
# 1. 看绑定设备，记下 toyUid
qiui devices

# 2. 向云端要一条蓝牙写入数据（stdout 一行 hex）
HEX=$(qiui get-toy-token --toy-uid '<toyUid>')
# 或关锁类：
# HEX=$(qiui close-lock --toy-uid '<toyUid>')

# 3. 扫描附近设备（可选）
qiui scan --seconds 8

# 4. 写入设备（把 <MAC> 换成 scan 看到的地址）
qiui write --address <MAC> --hex "$HEX"
```

无真机时可用模拟写入验证命令是否跑通：

```bash
qiui write --mock --address AA:BB:CC:DD:EE:FF --hex "$HEX"
```

## 命令一览

| 命令 | 做什么 |
|------|--------|
| `discover-api` | 更新云端 API 地址 |
| `login` | 邮箱/手机密码登录并保存会话 |
| `whoami` | 查看本地已保存账号 |
| `devices` | 列出绑定玩具（需登录） |
| `get-toy-token` | 云端生成一条蓝牙写入 hex（需登录，`--toy-uid`） |
| `close-lock` | 云端生成关锁类蓝牙 hex（需登录，`--toy-uid`） |
| `decry-notify` | 用云端解密设备回包 hex（需登录，`--hex`） |
| `profiles` | 列出蓝牙 profile 名称 |
| `scan` | 扫描蓝牙设备（`--seconds`，`--profile`，默认 `cellmate`） |
| `write` | 连接并写入 hex（`--address`、`--hex`；可选 `--wait-ms`、`--mock`、`--profile`） |
| `crypto …` | 本地加解密小工具（一般日常控设备用不到） |

全局：

```bash
qiui --verbose <命令>
qiui --token '…' devices
```

不确定参数时：`qiui <命令> --help`。

## 常见问题

| 情况 | 做法 |
|------|------|
| 提示没有 token | `qiui login …` |
| 连不上 API | `qiui discover-api`，或 `--base https://appapi.qiuitoy.com` |
| 找不到 `qiui` | 检查安装目录是否在 `PATH`，重开终端 |
| 没有蓝牙硬件 | 用 `write --mock`；真机需系统蓝牙可用 |

更多示例见：https://github.com/CntierTeam/QIUI/blob/main/README.md  
下载页：https://github.com/CntierTeam/QIUI/releases
