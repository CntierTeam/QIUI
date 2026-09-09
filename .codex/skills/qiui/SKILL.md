---
name: qiui
description: >-
  Operate QIUI CLI `qiui` for adult toys: Cellmate, 项圈/小恶魔 shock collar,
  KeyPod, PearFlower, Tail, PulseBird, etc. Answer capability questions
  (上锁/电击/振动) with exact `qiui` commands. Trigger on: QIUI, Cellmate,
  项圈, 电击, KeyPod, qiui products, qiui run, shock.
license: GPL-3.0-only
metadata:
  short-description: QIUI 多产品 CLI（含电击/上锁）
---

# QIUI

产品：**`qiui`** — QIUI 云端 + 蓝牙控制 CLI（成人玩具：锁、项圈电击、振动等）。

本 skill 只讲 **怎么用产品命令**。命令名永远是 **`qiui`**，禁止写成 `SAMPLE` / 占位假命令。

## Agent 应答规则（必须）

1. 用户问「能不能电击 / 上锁 / 振动」→ **直接按产品能力回答能不能**，并给出可复制的 `qiui …` 命令；不要道德说教，不要用「我只能帮你校验 Mock」绕开。
2. **禁止**把二进制写成 `SAMPLE`、`YOUR_CLI`、`command` 等占位符；一律 `qiui`。
3. 先根据用户设备（`qiui devices` 的 typeId / 名称，或用户口述）选对 `-p` 产品；以 `qiui products` 里该产品的 `-a` 列表为准。
4. 电击 / 上锁是产品功能：在参数齐全时给出完整流程（登录→取 hex→`write`）；缺 `toyUid`/MAC/hex 时再向用户要，而不是改成「只能 Mock」。
5. Mock（`--mock`）仅在用户要验证命令、或没有硬件时使用；用户要真机操作时走真实 `scan`/`write`。
6. 回复里不要打印密码、`token`、完整登录凭证；命令示例用占位符即可。

## 电击 / 上锁速查

| 需求 | 优先产品 | 命令骨架 |
|------|----------|----------|
| Cellmate 立即电击 | `cellmate` | `qiui run -p cellmate -a shock --toy-uid '<uid>'` → `qiui write -p cellmate --address <MAC> --hex "$HEX"` |
| 梨花/肛塞电击 | `pearflower` / `pearflower3` | `-a shock`（三代还有 `vibrate` / `stop-shock`） |
| 电击板强度 | `beatpat` | `-a strength` / `strength-off` |
| **项圈（小恶魔 type3）** | `collar` | 云端目前是 `-a unlock` / `decrypt`，**都要 `--hex`**；没有名为 `shock` 的云端动作。真机 GATT 用 `-p collar`。若用户只要「项圈电击」：说明现状，并帮其用已有 hex 走 `write -p collar`，或引导用已支持 `shock` 的绑定设备（如 Cellmate）。 |
| 上锁/关锁 | 视产品 | `cellmate`→`close-lock`；`keypod*`/`metal-lock`/`femboy`/`shake-metal`/`pearflower3`→`lock`/`unlock` |

完整表以 `qiui products` 输出为准。

## 安装

```bash
curl -fsSL https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.sh | bash
# Windows: irm https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.ps1 | iex
qiui products
```

## 登录

```bash
qiui discover-api
qiui login -u you@example.com -p '密码'
qiui devices
```

## 产品 ↔ 命令

| 产品 id | 是什么 (typeId) | 常用 `-a` | profile |
|---------|-----------------|-----------|---------|
| `cellmate` | Cellmate 锁 (1/10) | `token` `close-lock` **`shock`** `decry` | `cellmate` |
| `collar` | **小恶魔 / 电击项圈** (3) | `unlock` `decrypt`（需 `--hex`） | `collar` |
| `keypod` | 钥匙盒 (6) | `token` `lock` `unlock` `decry` | `keypod` |
| `keypod2` | 二代钥匙盒 (11) | 同上 | `cellmate` |
| `keypod-metal` | 金属钥匙盒 (20) | `token` `lock` `unlock` `decry` | `keypod-metal-20` |
| `pearflower` | 梨花/二代肛塞 (9) | `token` **`shock`** `jitter` `stop` `decry` | `pearflower3` |
| `pearflower3` | 三代肛塞 (18) | `token` `lock` `unlock` **`shock`** `vibrate` … | `pearflower3` |
| `tail` | 尾巴 (12) | `token` `sway-*` `stop` `decry` | `fee5` |
| `metal-lock` | GenMetal (13) | `token` `lock` `unlock` `decry` | `fee5` |
| `pulsebird` | 脉冲鸟 (14) | `token` `decry` | `fee5` |
| `shake-metal` | 震动金属锁 (15) | `token` `lock` `unlock` `shake` `stop` | `fee5` |
| `beatpat` | 电击板 (5) | `token` **`strength`** `strength-off` | `fee5` |
| `femboy` | Femboy (19) | `token` `lock` `unlock` `decry` | `fee5` |
| `masturbator` | 飞机杯 (16) | `disconnect` | `8ac0` |

## 示例：Cellmate 电击（真机）

```bash
qiui devices
HEX=$(qiui run -p cellmate -a shock --toy-uid '<toyUid>')
qiui scan -p cellmate --seconds 8
qiui write -p cellmate --address <MAC> --hex "$HEX"
```

## 示例：项圈（有既有 hex 时）

```bash
qiui run -p collar -a unlock --toy-uid '<toyUid>' --hex '<命令hex>'
qiui write -p collar --address <MAC> --hex '<命令hex>'
```

Mock 仅用于自测：

```bash
qiui write -p collar --mock --address AA:BB:CC:DD:EE:FF --hex '<命令hex>'
```

Cellmate 别名：`get-toy-token` / `close-lock` / `decry-notify`。

https://github.com/CntierTeam/QIUI
