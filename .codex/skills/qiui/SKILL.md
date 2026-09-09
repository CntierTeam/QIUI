---
name: qiui
description: >-
  Product usage for QIUI CLI `qiui`: Cellmate lock, KeyPod, PearFlower, GenMetal,
  PulseBird, BeatPat, electric collar (项圈). Trigger on: QIUI, Cellmate, KeyPod,
  PearFlower, 项圈, PulseBird, get-toy-token, qiui products, qiui run.
license: GPL-3.0-only
metadata:
  short-description: QIUI 多产品 CLI 用法
---

# QIUI

产品：**`qiui`** — QIUI 云端账号 + 多设备蓝牙控制 CLI。

本 skill **只讲怎么用命令**，不讲源码/协议开发。

## 安装

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.sh | bash
```

```powershell
# Windows
irm https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.ps1 | iex
```

```bash
qiui --help
qiui products    # 看支持哪些产品与动作
```

## 登录

```bash
qiui discover-api
qiui login -u you@example.com -p '密码'
qiui login -u 13800138000 -p '密码' --phone
qiui whoami
qiui devices          # 绑定列表 → 记下 toyUid / 蓝牙地址
```

会话：`~/.config/qiui/config.toml`；可用 `QIUI_TOKEN` / `--token` 覆盖。

## 产品一览

先跑 `qiui products`。当前云端动作覆盖：

| 产品 id | 是什么 | 常用动作 | 默认 BLE profile |
|---------|--------|----------|------------------|
| `cellmate` | Cellmate 贞操锁 Gen2/Gen3 | `token` `close-lock` `shock` `decry` | `cellmate` |
| `keypod-metal` | KeyPod Metal 金属钥匙舱 | `token` `lock` `unlock` `decry` | `fee5` |
| `keypod` | KeyPod / KeyPod Pro | `token` `lock` `unlock` `decry` | `fee5` |
| `pearflower3` | PearFlower Three | `token` `lock` `unlock` `shock` `vibrate` `stop-shock` `stop-vibrate` `decry` | `pearflower3` |
| `pearflower` | PearFlower 旧款 | `token` `shock` `jitter` `stop` `decry` | `pearflower3` |
| `shake-metal` | GenMetal 震动金属锁 | `token` `lock` `unlock` `shake` `stop` `decry` | `ae3` |
| `metal-lock` | Metal Lock | `token` `lock` `unlock` `decry` | `ae3` |
| `pulsebird` | PulseBird | `token` `decry` | `fee5` |
| `beatpat` | BeatPat / StrikePad | `token` `strength` `strength-off` `decry` | `8ac0` |
| `collar` | **电击项圈** | `unlock` `decrypt`（都要 `--hex`） | `fee5` |

`decry` / 项圈动作需要把设备回包 hex 用 `--hex` 交给云端。

## 控设备流程

```bash
# 1) 云端要一条可写入的 hex
HEX=$(qiui run -p cellmate -a token --toy-uid '<toyUid>')
HEX=$(qiui run -p cellmate -a close-lock --toy-uid '<toyUid>')
HEX=$(qiui run -p cellmate -a shock --toy-uid '<toyUid>')

HEX=$(qiui run -p keypod-metal -a lock --toy-uid '<toyUid>')
HEX=$(qiui run -p pearflower3 -a vibrate --toy-uid '<toyUid>')
HEX=$(qiui run -p shake-metal -a shake --toy-uid '<toyUid>')

# 2) 扫描（按产品选 profile）
qiui scan -p cellmate --seconds 8
qiui scan -p collar --seconds 8

# 3) 写入
qiui write -p cellmate --address <MAC> --hex "$HEX"
# 无硬件冒烟：
qiui write -p cellmate --mock --address AA:BB:CC:DD:EE:FF --hex "$HEX"
```

### 电击项圈

项圈走 `electricShockRecord`：把本地/回包 hex 交给云端。

```bash
qiui run -p collar -a unlock --toy-uid '<toyUid>' --hex '<命令hex>'
qiui run -p collar -a decrypt --toy-uid '<toyUid>' --hex '<命令hex>'
```

### Cellmate 快捷别名

```bash
qiui get-toy-token --toy-uid '<toyUid>'     # = run -p cellmate -a token
qiui close-lock --toy-uid '<toyUid>'        # = run -p cellmate -a close-lock
qiui decry-notify --hex '<notify_hex>'      # = run -p cellmate -a decry --hex …
```

## 命令一览

| 命令 | 做什么 |
|------|--------|
| `discover-api` / `login` / `whoami` / `devices` | 账号与绑定 |
| `products` | 列出产品与动作 |
| `run -p … -a … --toy-uid …` | 云端动作 → hex（或 JSON） |
| `scan` / `write` | 蓝牙扫/写；`-p <产品>` 选 profile |
| `profiles` | 底层 GATT profile 名 |
| `crypto …` | 本地加解密小工具 |

全局：`qiui --verbose …`、`qiui --token '…' …`。不确定就 `qiui <命令> --help`。

## 常见问题

| 情况 | 做法 |
|------|------|
| 缺 token | `qiui login …` |
| 不知道产品 id | `qiui products` |
| write 连不上特征 | 换对的 `-p`（产品）或 `--profile` |
| 无蓝牙硬件 | `write --mock` |

README：https://github.com/CntierTeam/QIUI  
Release：https://github.com/CntierTeam/QIUI/releases
