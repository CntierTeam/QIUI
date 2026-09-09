---
name: qiui
description: >-
  Product usage for QIUI CLI `qiui`: Cellmate, 项圈/小恶魔, KeyPod, PearFlower,
  Tail, GenMetal, PulseBird, ShockGenMetal, BeatPat, Femboy. Trigger on: QIUI,
  Cellmate, KeyPod, 项圈, PulseBird, PearFlower, qiui products, qiui run.
license: GPL-3.0-only
metadata:
  short-description: QIUI 多产品 CLI 用法
---

# QIUI

产品：**`qiui`** — QIUI 云端账号 + 多设备蓝牙控制 CLI。

本 skill **只讲怎么用命令**。

## 安装

```bash
curl -fsSL https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.sh | bash
```

```powershell
irm https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.ps1 | iex
```

```bash
qiui products    # 必看：产品 id + 动作
qiui profiles    # GATT profile 名
```

## 登录

```bash
qiui discover-api
qiui login -u you@example.com -p '密码'
qiui devices          # 记下 toyUid；typeId 对应产品
```

## 产品 ↔ 命令

以 `qiui products` 为准。常用对照：

| 产品 id | 是什么 (typeId) | 常用 `-a` | 默认 `-p` profile |
|---------|-----------------|-----------|-------------------|
| `cellmate` | Cellmate 锁 (1/10) | `token` `close-lock` `shock` `decry` | `cellmate` |
| `collar` | **小恶魔 / 电击项圈** (3) | `unlock` `decrypt`（都要 `--hex`） | `collar` |
| `keypod` | 钥匙盒 (6) | `token` `lock` `unlock` `decry` | `keypod` |
| `keypod2` | 二代钥匙盒 (11) | 同上 | `cellmate` |
| `keypod-metal` | 金属钥匙盒 (20) | `token` `lock` `unlock` `decry` | `keypod-metal-20` |
| `pearflower` | 梨花/二代肛塞 (9) | `token` `shock` `jitter` `stop` `decry` | `pearflower3` |
| `pearflower3` | 三代肛塞 (18) | `token` `lock` `unlock` `shock` `vibrate` … | `pearflower3` |
| `tail` | 尾巴 (12) | `token` `sway-*` `stop` `decry` | `fee5` |
| `metal-lock` | GenMetal 金属锁 (13) | `token` `lock` `unlock` `decry` | `fee5` |
| `pulsebird` | 脉冲鸟 (14) | `token` `decry` | `fee5`（**不是** ffa0） |
| `shake-metal` | 震动金属锁 (15) | `token` `lock` `unlock` `shake` `stop` | `fee5` |
| `beatpat` | 电击板 (5) | `token` `strength` `strength-off` | `fee5`（GATT 可能空） |
| `femboy` | Femboy/SissyStar (19) | `token` `lock` `unlock` `decry` | `fee5` |
| `masturbator` | 飞机杯 (16) | `disconnect` | `8ac0` |

ThrillCage (type 21，GATT=`ffa0`) 目前只在 `qiui profiles` 有 profile，云端→hex 未接。`ffa0` **不是** PulseBird。

## 控设备

```bash
HEX=$(qiui run -p cellmate -a token --toy-uid '<toyUid>')
HEX=$(qiui run -p collar -a unlock --toy-uid '<toyUid>' --hex '<命令hex>')
HEX=$(qiui run -p keypod -a lock --toy-uid '<toyUid>')
HEX=$(qiui run -p pulsebird -a token --toy-uid '<toyUid>')

qiui scan -p collar --seconds 8
qiui write -p collar --address <MAC> --hex "$HEX"
qiui write -p cellmate --mock --address AA:BB:CC:DD:EE:FF --hex "$HEX"
```

Cellmate 别名：`get-toy-token` / `close-lock` / `decry-notify`。

## 命令一览

| 命令 | 做什么 |
|------|--------|
| `login` / `devices` | 账号与绑定 |
| `products` | 产品与动作 |
| `run -p … -a … --toy-uid …` | 云端 → hex |
| `scan` / `write` | 蓝牙；`-p` 选产品 profile |

不确定就 `qiui <命令> --help`。

https://github.com/CntierTeam/QIUI
