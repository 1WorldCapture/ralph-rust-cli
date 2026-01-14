# Ralph CLI（`ralph`）产品需求文档（PRD）

## 1. 背景与现状

当前项目在 `scripts/` 目录下以两个 Bash 脚本作为工具入口：

- 单次运行：`scripts/ralph-once.sh`
- 循环运行：`scripts/ralph-loop.sh`

脚本用于调用不同的 Agent/AI Provider CLI（`droid`、`codex`、`claude`、`gemini`），并向其注入一段固定的 System Prompt（指导 Agent 使用 `bd` 做任务管理、按流程推进、运行质量门禁、提交/关闭任务等）。

随着功能迭代，Shell 脚本存在以下问题：

- 不便于版本发布与升级（用户需要手动更新脚本/重新安装）。
- 不便于分发与部署（难以作为标准二进制放入系统 `bin` 目录统一管理）。
- 逻辑与配置耦合（System Prompt 与 Provider 支持变化时，需要改脚本，维护成本高）。

## 2. 目标与范围

### 2.1 目标

- 用 Rust 重构为一个可分发的 CLI 程序，最终可执行文件名为：`ralph`。
- 功能与现有两个脚本对齐（行为一致），并提升安装、升级与运行体验。
- 将可变部分（System Prompt）抽离为用户可编辑的 Markdown 配置文件，降低迭代成本。
- 支持版本发布与版本管理，确保每次新增功能可对应一个可追踪版本。
- 提供一个自动升级命令，减少用户手动重新下载安装的成本。

### 2.2 非目标（本 PRD 不覆盖）

- 不定义/不更改 `bd`（beads）本身的功能与工作流；`ralph` 仅负责调用外部 Provider CLI 并传入 Prompt。
- 不在本 PRD 内确定最终分发渠道与包管理方式的唯一方案（例如 Homebrew、Cargo、GitHub Releases 等）；但需满足“可发布版本 + 可升级”的产品体验，具体实现由技术设计（TDD）定稿。

## 3. 核心概念

- **Provider**：外部 Agent/AI 命令行工具（如 `droid`、`codex`、`claude`、`gemini`）。`ralph` 负责选择 provider、拼装参数、执行命令并处理输出。
- **System Prompt**：传入 Provider 的系统级提示词，指导其如何在当前仓库内开展任务（例如使用 `bd`、按步骤推进等）。
- **配置目录**：用户 HOME 下的 `~/.Ralph/`，用于存放默认/自定义的 System Prompt 文件等用户侧配置。

## 4. 功能需求（Functional Requirements）

### 4.1 CLI 入口与命令结构

- 程序名：`ralph`
- 需要覆盖脚本的两种运行模式：一次性执行与循环执行。
- 需要提供升级命令：`ralph upgrade`（命名选择 `upgrade`，语义为“将当前安装升级到新版本”）。

> 具体子命令形式可采用 `ralph once` / `ralph loop`（或与现有脚本名称一致的别名）。最终命令行结构以实现阶段的技术设计为准，但必须覆盖等价能力与参数。

### 4.2 Provider 支持与参数对齐

- 默认 Provider：`droid`
- 支持的 Provider（首版至少包含脚本中的集合）：`droid`、`codex`、`claude`、`gemini`
- Provider 参数校验：
  - 对非法 provider 需给出明确错误信息并以非 0 退出码退出。
  - `--help`/`-h` 输出清晰的用法与示例。
- Provider 调用行为对齐（与脚本一致）：
  - `droid`：调用 `droid exec ...`（包含脚本当前使用的关键 flags）。
  - `codex`：调用 `codex exec ...`
  - `claude`：调用 `claude ...`
  - `gemini`：调用 `gemini ...`

> `ralph` 不负责实现 Provider 本身，只负责作为“调度器”统一入口。

### 4.3 单次运行（对齐 `ralph-once.sh`）

- 允许用户选择 provider（默认 `droid`）。
- 读取 System Prompt（见 4.5）并传给 provider 执行一次。
- 输出 provider 的结果到标准输出（stdout），并保持退出码语义可用于脚本/自动化集成。

### 4.4 循环运行（对齐 `ralph-loop.sh`）

- 允许用户选择 provider（默认 `droid`）。
- 允许用户指定迭代次数 `--iterations <count>`（默认 `10`）：
  - 需校验为正整数，否则报错并非 0 退出。
- 循环执行：
  - 每次迭代调用 provider 执行，并获取其输出。
  - 若输出包含字符串 `<promise>COMPLETE</promise>`，则提前结束循环。
  - 循环结束（提前结束或跑满迭代数）后，执行 `bd list --pretty` 输出任务状态概览（与脚本一致）。

### 4.5 System Prompt 抽取为 Markdown 配置文件

- 默认配置目录：`~/.Ralph/`
- 默认 System Prompt 文件（Markdown）：建议命名为 `system-prompt.md`（文件名可在实现阶段最终确定，但需稳定且可文档化）。
- 首次安装/首次运行行为：
  - 若 `~/.Ralph/` 不存在：自动创建。
  - 若默认 prompt 文件不存在：自动生成并写入默认内容（等价于当前脚本中的 PROMPT 文本）。
- 运行时行为：
  - `ralph` 运行时从该 Markdown 文件读取内容作为 System Prompt。
  - 用户可直接编辑该文件以自定义 System Prompt；`ralph` 无需重新编译即可生效。

### 4.6 版本发布与版本管理

- 项目需要遵循明确的版本号机制（例如语义化版本 SemVer：`MAJOR.MINOR.PATCH`）。
- `ralph` 需要提供查看当前版本的能力（例如 `ralph --version` / `ralph version`），用于排障与升级判断。
- 每次发布应生成可分发产物（可执行文件/包），并能追踪对应变更记录（如 `CHANGELOG` 或 release notes）。

### 4.7 自动升级（`ralph upgrade`）

- 命令：`ralph upgrade`
- 行为目标：用户安装后，后续可通过该命令将本地 `ralph` 升级到官方发布的最新版本，避免手动重装。
- 交互要求：
  - 展示当前版本与目标版本（若可获取）。
  - 升级过程给出明确进度/结果提示。
  - 若因权限不足无法写入目标路径，应给出可操作的解决方案（例如提示用更合适的安装方式或以管理员权限执行）。

> 升级的具体分发来源（例如 GitHub Releases、包管理器、内部制品库）与校验策略（校验和/签名）由技术设计阶段明确，但 PRD 要求最终用户体验为“一条命令完成升级”。

## 5. 用户故事（按依赖顺序）

以下用户故事按照“先完成上游能力，才能完成下游能力”的依赖顺序排列。每个用户故事均包含：

- (a) 要做的事情（用户动机/场景）
- (b) 要实现的功能（系统行为/范围）
- (c) 如何验收（可执行、可判断的验收标准）

### US-01：作为维护者，我要能发布带版本号的 `ralph`

- (a) 要做的事情  
  作为维护者，我希望在新增功能或修复问题后能够发布一个新版本，让用户拿到可追踪、可回滚的稳定产物。
- (b) 要实现的功能  
  - 项目具备明确的版本号（例如 SemVer）并能随着发布递增。
  - 发布产物中能体现版本信息，且 `ralph` 在运行时可输出自身版本。
  - 具备发布所需的最小规范（例如变更记录或 release notes 的生成/维护方式）。
- (c) 如何验收  
  - 在本地构建出的 `ralph` 执行 `--version`（或等价命令）能输出版本号。
  - 发布流程能产出与该版本号一致的分发产物（例如可执行文件或安装包）。
  - 版本号变更后，重新构建/发布的产物版本号同步变化且可被用户辨识。

**依赖**：无（基础能力）。

### US-02：作为用户，我首次使用时自动获得可编辑的 System Prompt 文件

- (a) 要做的事情  
  作为用户，我希望安装后无需手动创建配置目录与文件，就能马上开始使用，并能随时编辑提示词来适配我的工作流。
- (b) 要实现的功能  
  - 首次运行 `ralph` 时，自动创建 `~/.Ralph/`。
  - 自动生成默认的 Markdown Prompt 文件（内容等价于脚本内置 PROMPT）。
  - 后续运行从该文件读取 Prompt。
- (c) 如何验收  
  - 删除（或在全新环境中不存在）`~/.Ralph/` 后运行一次 `ralph`，会自动生成 `~/.Ralph/` 以及默认 prompt 文件。
  - prompt 文件为纯文本 Markdown，可用编辑器修改并保存。
  - 修改 prompt 文件后再次运行 `ralph`，传入 provider 的提示词内容发生对应变化（可通过调试输出或 provider 侧可观测行为验证）。

**依赖**：US-01（版本化发布可并行，但通常先具备基本 CLI 框架与可运行产物）。

### US-03：作为用户，我可以单次执行一次“任务推进”调用

- (a) 要做的事情  
  作为用户，我希望像现在 `ralph-once.sh` 一样，一次性触发 provider 执行，让我观察输出并做下一步决策。
- (b) 要实现的功能  
  - 提供“单次执行”入口（等价于现有脚本 once 的能力）。
  - 支持 `--provider <name>`，默认 `droid`，并校验合法性。
  - 从 `~/.Ralph/` 读取 System Prompt，并将其传给 provider 执行一次。
  - 标准输出打印 provider 输出；退出码可用于自动化判断成功/失败。
- (c) 如何验收  
  - 运行 `ralph ... --provider droid`（或等价 once 子命令）会实际调用到 `droid` 并产生输出。
  - 对非法 provider 返回非 0 退出码并输出明确错误提示。
  - 不带 `--provider` 时默认使用 `droid`。

**依赖**：US-02（需要可用的 System Prompt 文件与读取逻辑）。

### US-04：作为用户，我可以循环执行直到任务完成或达到迭代上限

- (a) 要做的事情  
  作为用户，我希望像现在 `ralph-loop.sh` 一样，让 `ralph` 自动重复调用 provider，直到 provider 输出明确的完成标记或达到迭代次数上限。
- (b) 要实现的功能  
  - 提供“循环执行”入口（等价于现有脚本 loop 的能力）。
  - 支持 `--provider`（默认 `droid`）与 `--iterations`（默认 `10`，必须为正整数）。
  - 每次迭代执行 provider，并检查输出是否包含 `<promise>COMPLETE</promise>`：
    - 若包含：提前退出循环。
    - 若不包含：继续下一次迭代，直到达到上限。
  - 循环结束后执行 `bd list --pretty` 并输出其结果。
- (c) 如何验收  
  - `--iterations 5` 时最多执行 5 次 provider 调用（可通过日志/输出计数验证）。
  - 当某次输出包含 `<promise>COMPLETE</promise>` 时，会在该次后停止继续迭代，并输出“完成/结束”的可见提示与 `bd list --pretty` 的结果。
  - `--iterations 0` 或非数字会报错并以非 0 退出码退出。

**依赖**：US-03（需要稳定的 provider 执行能力与 prompt 注入能力）。

### US-05：作为用户，我可以通过 `ralph upgrade` 一键升级到最新版本

- (a) 要做的事情  
  作为用户，我希望在有新版本发布时不需要手动下载与替换二进制，只需运行一条命令即可完成升级。
- (b) 要实现的功能  
  - 提供 `ralph upgrade` 命令，自动检测最新版本并完成升级安装。
  - 升级前后能展示当前版本与升级后的版本。
  - 当无法写入安装位置（权限/路径问题）时，给出明确且可操作的解决方案提示。
- (c) 如何验收  
  - 在存在“更新版本”的可用分发源时，执行 `ralph upgrade` 后，`ralph --version` 输出应从旧版本变为新版本。
  - 在无更新时，`ralph upgrade` 应提示“已是最新版本”（或等价表述）并保持退出码为成功（0）。
  - 当写入失败时，命令应失败并输出明确原因与下一步操作建议。

**依赖**：US-01（必须有可识别的版本与发布产物，升级才有意义）；建议在 US-03/US-04 稳定后实现以降低升级后行为变化风险。

## 6. 开放问题（实现前需确认）

- 分发与升级渠道：是否以 GitHub Releases 为默认，或以包管理器为主（Homebrew/Cargo），以及升级命令具体如何实现“自动获取与替换”。
- `~/.Ralph/` 中除 `system-prompt.md` 外，是否需要预留 Provider 配置（例如不同 provider 的命令模板/flags）以便未来动态扩展。
- CLI 结构最终形态：`ralph once/loop` vs `ralph run/loop` vs 兼容 `ralph-once`/`ralph-loop` 的别名策略。
