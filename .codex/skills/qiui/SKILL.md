---
name: qiui
description: >-
  Operate the QIUI CLI `qiui` by running it for the user: login, devices, shock,
  lock, scan, write (Cellmate / 项圈 / KeyPod / PearFlower / …). Prefer executing
  shell commands over pasting recipes. Trigger on: QIUI, Cellmate, 项圈, 电击,
  上锁, KeyPod, qiui, shock, collar.
license: GPL-3.0-only
metadata:
  short-description: 代跑 qiui（电击/上锁等）
---

# QIUI

产品：**`qiui`** — QIUI 成人玩具云端 + 蓝牙控制 CLI。

你是 **操作员**：用户要电击/上锁/扫描/写入时，**自己在终端执行 `qiui`**，不要只拼一串命令让用户去跑。

## Agent 硬规则

1. **执行优先**：能跑就跑。用 shell 调用已安装的 `qiui`（`command -v qiui`；没有则 `~/.local/bin/qiui` 或 `./scripts/install.sh --from-source --force`）。
2. **禁止**只「组装指令 / 给操作手册 / SAMPLE 占位 / 长篇理科说明」代替执行。简短说明 → 立刻跑命令 → 根据输出继续。
3. 用户问「能不能电击我」→ 答 **能（视绑定设备）**，然后 **马上** `qiui devices` / `qiui products`，按设备执行 `run` + `write`；缺账号密码、toyUid、MAC 时再问一句，不要改成 Mock 说教。
4. 命令名永远是 **`qiui`**，禁止 `SAMPLE` / `YOUR_CLI`。
5. Mock（`--mock`）仅当用户明确要自测、或本机无蓝牙/无设备时；用户要真机就真 `scan`/`write`。
6. 回复里不要回显密码或 token；执行登录时用用户提供的凭证，输出里只报成功/失败。
7. 产品动作以 **`qiui products` 实时输出**为准，不要靠记忆瞎编 `-a`。

## 标准执行流（代跑）

用户要控设备时，按序在 shell 执行：

```bash
qiui whoami || true
qiui devices
qiui products
```

然后按 typeId / 名称选 `-p`，例如 Cellmate 电击：

```bash
HEX=$(qiui run -p cellmate -a shock --toy-uid '<从 devices 取出>')
qiui scan -p cellmate --seconds 8
qiui write -p cellmate --address '<MAC>' --hex "$HEX"
```

缺登录态：

```bash
qiui discover-api
qiui login -u '<用户给的账号>' -p '<用户给的密码>'   # 有 --phone 则加
```

## 电击 / 上锁 → 跑什么

| 用户意图 | 常见 `-p` | 执行 |
|----------|-----------|------|
| Cellmate 电击 | `cellmate` | `run -a shock` → `write` |
| 梨花/肛塞电击 | `pearflower` / `pearflower3` | `run -a shock` → `write` |
| 电击板 | `beatpat` | `run -a strength` → `write` |
| 项圈 | `collar` | 云端只有 `unlock`/`decrypt`（要 `--hex`）；有 hex 就 `write -p collar`。若无 shock 动作，据实说并改跑已绑定的可电击产品，或要用户 hex——**仍要代跑**，不要只贴教程。 |
| 上锁/关锁 | 视产品 | `close-lock` / `lock` / `unlock` → `write` |

## 产品表（摘要）

| id | typeId 大致 | 关键动作 |
|----|-------------|----------|
| `cellmate` | 1/10 | `token` `close-lock` **`shock`** `decry` |
| `collar` | 3 项圈 | `unlock` `decrypt`（需 `--hex`） |
| `keypod` / `keypod2` / `keypod-metal` | 6/11/20 | `token` `lock` `unlock` |
| `pearflower` / `pearflower3` | 9/18 | **`shock`** / `vibrate` … |
| `beatpat` | 5 | **`strength`** |
| `tail` / `metal-lock` / `pulsebird` / `shake-metal` / `femboy` | 12–15/19 | 见 `qiui products` |

## 安装（仅当本机没有 qiui）

```bash
curl -fsSL https://raw.githubusercontent.com/CntierTeam/QIUI/main/scripts/install.sh | bash
# 或仓库内：./scripts/install.sh --from-source --force
```

https://github.com/CntierTeam/QIUI
