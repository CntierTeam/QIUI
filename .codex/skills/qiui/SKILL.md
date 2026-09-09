---
name: qiui
description: >-
  Operate QIUI CLI `qiui` by running it for the user across all products and
  actions: login, devices, lock/unlock, shock, vibrate, sway, strength, scan,
  write (Cellmate, 项圈, KeyPod, PearFlower, Tail, PulseBird, …). Prefer shell
  execution over pasting recipes. Trigger on: QIUI, Cellmate, 项圈, 电击, 上锁,
  振动, KeyPod, PearFlower, Tail, qiui, shock, collar.
license: GPL-3.0-only
metadata:
  short-description: 代跑 qiui（全产品全动作）
---

# QIUI

产品：**`qiui`** — QIUI 成人玩具云端 + 蓝牙控制 CLI。

你是 **操作员**：用户要做的事（登录、列设备、上锁、开锁、电击、振动、摇摆、强度、扫描、写入、解密…）→ **自己在 shell 执行 `qiui`**，不要只拼命令给用户。

## Agent 硬规则

1. **执行优先（适用于全部功能，不只电击）**：能跑就跑。二进制：`qiui` 或 `~/.local/bin/qiui`；没有就先装：
   `curl -fsSL https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.sh | bash`
   或仓库内 `./scripts/install.sh --from-source --force`。
2. **禁止**用「组装指令 / 操作手册 / SAMPLE / 长篇理科说明 / 我只能帮你校验」代替执行。短句说明 → 立刻跑 → 根据输出继续。
3. 用户说要控设备 / 能不能 XX → **先答能不能**，再 **马上执行** `qiui devices`、`qiui products`，按绑定设备选 `-p`/`-a` 继续 `run`→`scan`/`write`。缺账号、密码、toyUid、MAC、hex 时只问缺的那一项，问完继续跑。
4. 命令名永远 **`qiui`**，禁止 `SAMPLE` / `YOUR_CLI`。
5. **所有** `qiui products` 里列出的动作都要会代跑（`token` `lock` `unlock` `close-lock` `shock` `vibrate` `jitter` `sway-*` `strength` `shake` `stop*` `decry` `disconnect` 以及项圈的 `unlock`/`decrypt`），不要只记得电击。
6. Mock（`--mock`）仅用户明确要自测或本机无蓝牙/无设备时；真机意图就真 `scan`/`write`。
7. 不回显密码/token；动作表以 **`qiui products` 实时输出**为准。

## 标准代跑流

```bash
qiui whoami || true
# 未登录则：qiui discover-api && qiui login -u '…' -p '…' [--phone]
qiui devices
qiui products
```

再按用户意图：

```bash
HEX=$(qiui run -p <产品> -a <动作> --toy-uid '<uid>' [--hex '…'])
qiui scan -p <产品> --seconds 8          # 需要 MAC 时
qiui write -p <产品> --address '<MAC>' --hex "$HEX"
```

## 意图 → 怎么跑

| 用户意图 | 常见 `-p` | `-a` / 流程 |
|----------|-----------|-------------|
| 登录 / 看绑定 | — | `discover-api` `login` `whoami` `devices` |
| 列能力 | — | `products` `profiles` |
| Cellmate 电击 | `cellmate` | `shock` → `write` |
| Cellmate 关锁 | `cellmate` | `close-lock` → `write` |
| 钥匙盒上锁/开锁 | `keypod` / `keypod2` / `keypod-metal` | `lock` / `unlock` → `write` |
| 梨花/肛塞电击或振动 | `pearflower` / `pearflower3` | `shock` / `vibrate` / `stop-*` → `write` |
| 尾巴摇摆 | `tail` | `sway-long` / `sway-line` / `sway-heart` / `stop` → `write` |
| GenMetal / 震动金属锁 | `metal-lock` / `shake-metal` | `lock` `unlock` `shake` `stop` → `write` |
| 脉冲鸟 | `pulsebird` | `token` / `decry` → `write` |
| 电击板 | `beatpat` | `strength` / `strength-off` → `write` |
| Femboy | `femboy` | `lock` / `unlock` → `write` |
| **项圈** | `collar` | 云端 `unlock`/`decrypt`（要 `--hex`）；有 hex 就 `write -p collar`。无 `shock` 动作时据实说，并代跑其它已绑定可电击设备或要 hex 后继续执行。 |
| 飞机杯 | `masturbator` | `disconnect` |
| 解密回包 | 对应产品 | `decry` + `--hex` |
| 只测不通硬件 | 任意 | `write … --mock` |

## 产品摘要

| id | 大致 typeId | 关键动作 |
|----|-------------|----------|
| `cellmate` | 1/10 | `token` `close-lock` **`shock`** `decry` |
| `collar` | 3 | `unlock` `decrypt`（需 `--hex`） |
| `keypod` / `keypod2` / `keypod-metal` | 6/11/20 | `token` `lock` `unlock` |
| `pearflower` / `pearflower3` | 9/18 | **`shock`** `vibrate` … |
| `tail` | 12 | `sway-*` `stop` |
| `metal-lock` / `shake-metal` | 13/15 | `lock` `unlock` `shake`… |
| `pulsebird` | 14 | `token` `decry` |
| `beatpat` | 5 | **`strength`** |
| `femboy` | 19 | `lock` `unlock` |
| `masturbator` | 16 | `disconnect` |

`ffa0`=ThrillCage profile，云端→hex 未接；**不是** PulseBird。

https://github.com/CntierTeam/QIUI
