# Specite 最佳实践指南

本指南基于 Specite 的工作流设计，提供从项目初始化到迭代完成的全流程最佳实践，配以详细示例。

---

## 目录

1. [核心理念](#1-核心理念)
2. [安装与初始化](#2-安装与初始化)
3. [迭代生命周期总览](#3-迭代生命周期总览)
4. [阶段一：需求规格化 — `/spec`](#阶段一需求规格化--spec)
5. [阶段二：实施方案规划 — `/plan`](#阶段二实施方案规划--plan)
6. [阶段三：方案执行 — `/exec`](#阶段三方案执行--exec)
7. [阶段四：收尾交付 — `/post`](#阶段四收尾交付--post)
8. [快速通道 — `/iter`](#快速通道--iter)
9. [CLI 命令参考](#cli-命令参考)
10. [常见问题与排错](#常见问题与排错)

---

## 1. 核心理念

Specite 采用 **Spec-Driven（规格驱动）** 的迭代开发模式，将 AI 辅助开发拆解为可审查、可追溯的四个阶段：

```
/spec → /plan → /exec → /post
```

每个阶段都有明确的输入和输出物，避免"直接让 AI 写代码"导致的失控问题。核心理念：

- **先想清楚再动手**：通过 SPEC.md 锁定需求边界，通过 PLAN.md 分解实施步骤
- **阶段隔离**：每个阶段由专门的子代理（subagent）负责，主代理只做协调
- **可追溯**：所有迭代元数据记录在 `.specite/iters.json`，每次迭代的 SPEC、PLAN 文档独立保存

### 目录结构

```
your-project/
├── .specite/
│   ├── iters.json              # 迭代元数据（名称、阶段、时间戳）
│   ├── iterations/
│   │   └── add-user-auth/      # 每个迭代一个目录
│   │       ├── SPEC.md         # 需求规格文档
│   │       ├── PLAN.md         # 实施方案文档
│   │       └── FINISHED.md     # 完成报告（/post 生成）
│   ├── docs/                   # 外部库调研报告
│   │   ├── jwt-auth.md
│   │   └── bcrypt.md
│   └── templates/              # SPEC.md 和 PLAN.md 的模板
│       ├── SPEC.md
│       └── PLAN.md
├── .opencode/
│   ├── commands/               # slash 命令模板（specite 管理）
│   │   ├── spec.md
│   │   ├── plan.md
│   │   ├── exec.md
│   │   ├── iter.md
│   │   └── post.md
│   └── agents/                 # 子代理定义（specite 管理）
│       ├── phase-executor.md
│       ├── plan-creator.md
│       └── web-researcher.md
```

> **注意**：`.opencode/commands/`、`.opencode/agents/`、`.opencode/node_modules/` 已被 `.gitignore` 忽略，不需要提交到版本控制。

---

## 2. 安装与初始化

### 安装

```bash
# 推荐：npm 全局安装
npm install -g @fnnm/specite

# 或从源码编译
git clone https://github.com/fxcl/opencode-specite
cd opencode-specite
cargo install --path .
```

### 初始化项目

在项目根目录执行一次：

```bash
specite init
```

初始化会自动完成：

| 操作 | 说明 |
|------|------|
| 创建 `.specite/iterations/` | 存放每个迭代的 SPEC/PLAN 文档 |
| 创建 `.specite/docs/` | 存放外部库调研报告 |
| 创建 `.specite/templates/` | 复制 SPEC.md 和 PLAN.md 模板 |
| 创建 `.specite/iters.json` | 迭代元数据文件 |
| 安装 `.opencode/commands/` | 5 个 slash 命令模板 |
| 安装 `.opencode/agents/` | 3 个子代理定义 |
| 更新 `.gitignore` | 自动添加忽略条目 |
| `git init`（如果还不是 git 仓库） | 初始化版本控制 |

> **最佳实践**：在全新项目或已有项目中都可以安全运行 `specite init`，它是幂等的——重复运行只会更新管理文件，不会破坏已有迭代。

---

## 3. 迭代生命周期总览

每个迭代经历 5 个阶段（stage）：

```
new → specified → planned → executed → completed
 │       │           │          │          │
 │       │           │          │          └─ /post 完成
 │       │           │          └─ /exec 或 /iter 完成
 │       │           └─ /plan 或 /iter 完成
 │       └─ /spec 完成
 └─ specite new 创建
```

### 推荐工作流（稳定路径）

```
用户提出想法
    │
    ▼
  /spec          ← 与 AI 对话，澄清需求，生成 SPEC.md
    │
    ▼
  审查 SPEC.md    ← 人工检查，按需修改
    │
    ▼
  /plan 1        ← AI 自动生成 PLAN.md
    │
    ▼
  审查 PLAN.md    ← 人工检查，按需修改
    │
    ▼
  /exec 1        ← AI 按阶段逐步实现
    │
    ▼
  /post 1        ← 生成完成报告，提交 git
```

### 快速工作流（/iter 路径）

当需求明确、方案简单时，可跳过 `/plan` 的单独审查：

```
/spec → /iter 1   （自动规划 + 自动执行）
```

---

## 阶段一：需求规格化 — `/spec`

### 何时使用

- 有一个新的功能想法
- 需要修复一个复杂 bug（涉及多个文件或模块）
- 需要引入新的技术栈或外部库

### 操作步骤

#### 方式 A：先讨论，再规格化（推荐）

1. 在 OpenCode 中按 `Tab` 切换到 **Plan 模式**
2. 与 AI 自由讨论你的想法，澄清需求
3. 讨论充分后，切回 **Build 模式**
4. 运行 `/spec`（不带参数）

#### 方式 B：直接规格化

```
/spec 给项目添加 JWT 用户认证
```

### `/spec` 内部流程

执行 `/spec` 后，AI 会严格按照以下步骤工作：

```
步骤 0  读取 SPEC.md 模板
步骤 1  用 question 工具收集需求
        └─ 第一个问题始终是：迭代名称（建议 kebab-case）
步骤 2  运行 specite new <iteration-name> 创建迭代
步骤 3  委派 @explore 子代理探索现有代码
步骤 4  委派 @web-researcher 子代理调研外部库
        └─ 每个库一个子代理，报告保存到 .specite/docs/<topic>.md
步骤 5  综合所有信息，填写 SPEC.md
步骤 6  （如果没有 AGENTS.md）创建最小 AGENTS.md
```

### 示例：创建"添加 JWT 认证"迭代

用户在 OpenCode 中输入：
```
/spec 添加 JWT 用户认证，支持登录、注册、token 刷新
```

AI 会依次询问：
```
Q1: 迭代名称？（建议默认：add-jwt-auth）
Q2: 使用哪种 JWT 库？（如 jsonwebtoken）
Q3: token 过期策略？
Q4: 密码加密方案？（如 bcrypt）
Q5: 是否需要 RBAC 权限？
...
```

最终生成 `.specite/iterations/add-jwt-auth/SPEC.md`：

```markdown
# add-jwt-auth

## Goal

为项目添加完整的 JWT 用户认证系统，包括注册、登录、token 刷新和密码加密。

## Background

项目当前使用 Express.js，无认证机制。用户模型定义在 `src/models/User.ts`。

## Requirements

- 用户注册接口（POST /api/auth/register）
- 用户登录接口（POST /api/auth/login）
- Token 刷新接口（POST /api/auth/refresh）
- 密码使用 bcrypt 加密（cost factor 12）
- Access token 有效期 15 分钟，Refresh token 有效期 7 天

## Acceptance Criteria

- [ ] 注册接口返回 201 和用户信息（不含密码）
- [ ] 登录接口返回 access_token 和 refresh_token
- [ ] 刷新接口用有效 refresh_token 换取新的 access_token
- [ ] 密码在数据库中是 bcrypt 哈希
- [ ] 无效 token 返回 401
- [ ] 所有接口有输入验证

## Scope

- src/models/User.ts
- src/controllers/authController.ts（新建）
- src/middleware/auth.ts（新建）
- src/routes/auth.ts（新建）
- .env（添加 JWT_SECRET）

## Non-Goals

- OAuth 第三方登录
- 邮箱验证
- 密码重置流程

## Behavior Details

- 注册时邮箱唯一性校验
- 密码最少 8 个字符
- Token 格式：JWT HS256
- Refresh token 存储在 httpOnly cookie

## Dependencies And Research

- jsonwebtoken: https://github.com/auth0/node-jsonwebtoken
- bcrypt: https://github.com/kelektiv/node.bcrypt.js
- 调研报告: .specite/docs/jsonwebtoken.md, .specite/docs/bcrypt.md

## Verification

- npm test 运行全部测试
- 手动测试：curl 注册 → 登录 → 刷新 → 访问受保护路由

## Shifts

N/A
```

### 最佳实践

1. **迭代名称要语义化**：使用 kebab-case，如 `add-jwt-auth` 而非 `feature1`
2. **Non-Goals 和 Scope 同样重要**：明确划定边界能防止范围蔓延
3. **Acceptance Criteria 要可测试**：每条标准都应该是可以通过命令或观察验证的
4. **遇到不适用的小节写 `N/A`**：不要删除模板小节，保持结构一致性
5. **充分利用 @web-researcher**：涉及不熟悉的外部库时，让 AI 先调研再写 SPEC
6. **审查 SPEC.md**：`/spec` 完成后，务必打开 SPEC.md 检查，直接编辑修改不合理的部分

---

## 阶段二：实施方案规划 — `/plan`

### 何时使用

SPEC.md 创建完成并审查通过后。

### 操作

```
/plan 1
```

> `1` 指向最近创建或更新的迭代。运行 `specite list` 可查看迭代的序号。

### `/plan` 内部流程

```
步骤 0  读取 PLAN.md 模板
步骤 1  基于 SPEC.md 创建分阶段实施方案
        └─ 每个阶段包含：Status、目标、范围、步骤、验证方式、Completion Log
步骤 2  保存 PLAN.md
        └─ 运行 specite update <iter_id> planned
```

### 示例：JWT 认证的 PLAN.md

```markdown
# add-jwt-auth Plan

## Overview

分 4 个阶段实现 JWT 认证系统：数据模型 → 工具函数 → 控制器与路由 → 中间件与集成测试。

## Assumptions

- Express 项目已配置 body-parser 和 cors 中间件
- 数据库使用 MongoDB + Mongoose
- 环境变量 JWT_SECRET 已在 .env 中定义

## Phases

### Phase 1: 用户模型扩展

Status: `pending`

扩展 User 模型，添加 password 字段（bcrypt 哈希）和 pre-save 钩子。

实现步骤：
1. 在 src/models/User.ts 添加 password 字段（required, select: false）
2. 添加 pre-save 钩子，使用 bcrypt 哈希密码
3. 添加实例方法 comparePassword(candidate)

验证：单元测试 comparePassword 方法。

#### Completion Log

N/A

### Phase 2: JWT 工具函数

Status: `pending`

创建 JWT token 生成和验证工具函数。

实现步骤：
1. 创建 src/utils/jwt.ts
2. 实现 generateAccessToken(userId) → 返回 15 分钟有效的 JWT
3. 实现 generateRefreshToken(userId) → 返回 7 天有效的 JWT
4. 实现 verifyToken(token) → 返回解码后的 payload 或抛错

验证：单元测试 token 生成和验证。

#### Completion Log

N/A

### Phase 3: 认证控制器与路由

Status: `pending`

实现注册、登录、刷新三个接口。

实现步骤：
1. 创建 src/controllers/authController.ts
2. 实现 register：邮箱查重 → bcrypt 哈希 → 创建用户 → 返回 201
3. 实现 login：查用户 → 验密码 → 生成 token 对 → 设置 httpOnly cookie
4. 实现 refresh：验证 refresh token → 生成新 access token
5. 创建 src/routes/auth.ts，挂载三个路由
6. 在 src/app.ts 中挂载 /api/auth 路由

验证：curl 手动测试三个接口。

#### Completion Log

N/A

### Phase 4: 认证中间件与错误处理

Status: `pending`

实现 JWT 认证中间件，保护需要登录的路由。

实现步骤：
1. 创建 src/middleware/auth.ts
2. 实现 requireAuth 中间件：从 header 提取 Bearer token → 验证 → 挂载 req.user
3. 实现 requireRole(role) 中间件：检查 req.user.role
4. 统一错误处理：invalid token → 401，expired → 401
5. 在需要保护的路由上应用中间件

验证：集成测试覆盖认证流程。

#### Completion Log

N/A

## Cross-Phase Verification

- npm run lint 通过
- npm test 全部通过
- 完整流程测试：注册 → 登录 → 访问受保护路由 → 刷新 token

## Risks And Mitigations

- **风险**：bcrypt 在 serverless 环境性能差
  **缓解**：cost factor 设为 12（而非更高），并添加性能监控
- **风险**：refresh token 存储在 cookie 可能有 CSRF 风险
  **缓解**：添加 csurf 中间件

## Out Of Scope

N/A

## Changes

N/A
```

### 最佳实践

1. **阶段数量要适中**：通常 3-6 个阶段为宜，太多说明拆分过细，太少说明阶段粒度太大
2. **每个阶段必须独立可验证**：写明具体的验证命令或方法
3. **阶段顺序要合理**：基础层 → 业务层 → 集成层，避免循环依赖
4. **不要包含时间计划**：PLAN.md 不写 deadline 或里程碑日期，专注技术实施
5. **Completion Log 初始值为 `N/A`**：执行阶段后由 phase-executor 子代理填写
6. **审查 PLAN.md**：检查阶段划分是否合理，步骤是否具体，验证是否可操作

---

## 阶段三：方案执行 — `/exec`

### 何时使用

PLAN.md 创建完成并审查通过后。

### 操作

```
/exec 1
```

### `/exec` 内部流程

```
步骤 1  读取 PLAN.md，创建 todo list 跟踪所有阶段
步骤 2  逐个委派 @phase-executor 子代理
        └─ 每次只委派一个阶段（串行，避免编辑冲突）
步骤 3  等待当前阶段完成，再委派下一个
步骤 4  所有阶段完成后，向用户报告
步骤 5  运行 specite update <iter_id> executed
```

### 执行机制

- **主代理**：只做协调，不写代码
- **@phase-executor**：每个阶段一个独立子代理，负责读取 SPEC.md + PLAN.md，完成指定阶段的实现
- **串行执行**：阶段按顺序执行，前一个完成才开始下一个
- **可恢复**：如果中断，可以重新运行 `/exec`，已完成的阶段不会被重复执行

### 最佳实践

1. **执行期间不要手动编辑代码**：避免与子代理产生编辑冲突
2. **观察 todo list 进度**：主代理会维护一个 todo list，实时反映执行进度
3. **中断后可恢复**：如果 `/exec` 被中断，重新运行 `/exec 1` 即可继续
4. **检查结果**：执行完成后，检查 `git diff` 确认改动符合预期
5. **阶段验证**：每个 phase-executor 完成后会填写 Completion Log，可以在 PLAN.md 中查看

---

## 阶段四：收尾交付 — `/post`

### 何时使用

`/exec` 完成后，确认实现符合预期。

### 操作

```
/post 1
```

### `/post` 内部流程

```
步骤 1  获取 git status 和 git diff --stat
步骤 2  生成完成报告 FINISHED.md
步骤 3  运行 specite update <iter_id> completed
步骤 4  提交所有变更到 git
```

### 最佳实践

1. **确认工作完成**：在运行 `/post` 前，确保所有功能已实现并通过验证
2. **检查 git status**：确保没有意外的文件变更
3. **审查提交**：`/post` 会自动 commit，检查 commit message 是否准确
4. **首次迭代后创建 AGENTS.md**：第一次迭代完成后是创建 AGENTS.md 的好时机，为后续迭代提供项目指导

---

## 快速通道 — `/iter`

### 何时使用

当需求简单明确，不需要单独审查 PLAN.md 时，可以用 `/iter` 一步完成规划和执行。

### 操作

```
/iter 1
```

### `/iter` 内部流程

```
步骤 1  委派 @plan-creator 子代理创建 PLAN.md
步骤 2  等待 PLAN.md 完成
步骤 3  读取 PLAN.md，创建 todo list
步骤 4  逐个委派 @phase-executor 子代理执行阶段
步骤 5  所有阶段完成后，运行 specite update <iter_id> executed
```

### 与 `/plan` + `/exec` 的区别

| 对比项 | `/plan` + `/exec` | `/iter` |
|--------|--------------------|---------|
| 流程 | 分两步，可审查 PLAN | 一步到位 |
| PLAN.md | 用户可审查和修改 | 无人工审查环节 |
| 适用场景 | 复杂迭代，需要人工把关 | 简单迭代，方案明确 |
| 稳定性 | 高（推荐） | 实验性功能 |

### 最佳实践

- 首次使用 Specite 时，优先使用 `/plan` + `/exec` 的稳定路径
- 熟悉工作流后，对简单迭代（如添加一个工具函数）可以使用 `/iter`
- 复杂迭代（涉及架构变更、多模块重构）始终使用 `/plan` + `/exec`

---

## CLI 命令参考

除了在 OpenCode 中使用 slash 命令，specite 也可以直接在终端中使用。

### `specite init [path]`

初始化项目。

```bash
specite init                  # 初始化当前目录
specite init ~/projects/myapp # 初始化指定目录
```

### `specite new <name>`

创建新迭代。名称会自动转为 kebab-case。

```bash
specite new "Add User Auth"   # → add-user-auth
specite new fix-login-bug     # → fix-login-bug
```

### `specite list [limit]`

列出所有迭代，按最近更新排序。

```bash
specite list        # 列出全部
specite list 5      # 只显示最近 5 个
```

输出示例：
```
1. add-jwt-auth (specified)
2. fix-login-bug (completed)
3. refactor-api-routes (executed)
```

### `specite update <id|name> <stage>`

更新迭代阶段。有效阶段：`new`、`specified`、`planned`、`executed`、`completed`。

```bash
specite update 1 specified    # 使用序号
specite update add-jwt-auth planned  # 使用名称
```

### `specite prompt <target> [kind]`

生成命令提示（供 OpenCode slash 命令内部调用，通常不需要手动执行）。

```bash
specite prompt spec                    # 生成 /spec 提示
specite prompt 1 plan                  # 生成迭代 1 的 /plan 提示
specite prompt 1 exec                  # 生成迭代 1 的 /exec 提示
specite prompt 1 post                  # 生成迭代 1 的 /post 提示
```

### `specite path <id> <kind>`（内部辅助）

打印迭代文件的路径。

```bash
specite path 1 spec   # .specite/iterations/add-jwt-auth/SPEC.md
specite path 1 plan   # .specite/iterations/add-jwt-auth/PLAN.md
```

### `specite status <id>`（内部辅助）

打印迭代的当前阶段。

```bash
specite status 1   # specified
```

### 迭代 ID 说明

- 数字 `1` 始终指向最近创建或更新的迭代，`2` 指向倒数第二个，以此类推
- 也可以直接使用迭代名称：`specite update add-jwt-auth planned`
- 运行 `specite list` 确认序号映射

---

## 常见问题与排错

### Q: 运行 slash 命令时报 "No Specite project found"

**原因**：当前目录不在已初始化的 Specite 项目内。

**解决**：
```bash
cd /path/to/your/project
specite init
```

Specite 会向上递归查找 `.specite` 目录，因此只要在项目内的任意子目录运行命令都可以。

### Q: `/spec` 创建的 SPEC.md 不满意怎么办

直接编辑 `.specite/iterations/<name>/SPEC.md` 文件，修改后保存即可。`/plan` 会使用修改后的版本。

### Q: `/plan` 创建的 PLAN.md 需要调整

同上，直接编辑 `.specite/iterations/<name>/PLAN.md`。`/exec` 会使用修改后的版本。

### Q: `/exec` 执行中断了怎么办

直接重新运行 `/exec 1`。主代理会读取 PLAN.md 中各阶段的 Completion Log 来判断哪些已完成，从未完成的阶段继续。

### Q: 如何并行处理多个迭代

可以创建多个迭代，但建议同时只执行一个 `/exec`，因为子代理会编辑同一份代码库。

```bash
specite new "refactor-database"
specite new "add-logging"
specite list
# 1. add-logging (new)
# 2. refactor-database (new)
# 3. add-jwt-auth (completed)

/spec          # 处理最新的 add-logging
/plan 2        # 规划 refactor-database
```

### Q: `.specite/docs/` 目录什么时候会有文件

在 `/spec` 流程的第 4 步，当 AI 识别出需要研究的外部库时，会委派 `@web-researcher` 子代理进行调研，调研报告会保存为 `.specite/docs/<topic>.md`。这些报告会被 SPEC.md 引用，并在后续迭代中复用。

### Q: 迭代名称命名建议

| 场景 | 示例 |
|------|------|
| 新增功能 | `add-jwt-auth`、`add-export-csv` |
| 修复 Bug | `fix-login-redirect`、`fix-memory-leak` |
| 重构 | `refactor-api-routes`、`refactor-state-management` |
| 性能优化 | `optimize-db-queries`、`optimize-bundle-size` |

规则：
- 使用英文 kebab-case
- 动词开头：add/fix/refactor/optimize/update/remove
- 包含具体对象：jwt-auth 而非 auth，db-queries 而非 queries

### Q: AGENTS.md 的作用

AGENTS.md 是给 AI 代理的项目指导文件，包含项目概述和技术栈信息。`/spec` 流程会在首次迭代时自动创建一个最小版本。建议在第一次迭代完成后手动完善，为后续迭代提供更准确的上下文。

### Q: `.opencode/` 和 `.specite/` 的职责划分

| 目录 | 由谁管理 | 用途 |
|------|----------|------|
| `.opencode/commands/` | specite 安装，被 gitignore | slash 命令模板 |
| `.opencode/agents/` | specite 安装，被 gitignore | 子代理定义 |
| `.opencode/node_modules/` | OpenCode 运行时，被 gitignore | OpenCode 运行时依赖 |
| `.specite/iterations/` | specite 运行时 | 每次迭代的 SPEC/PLAN 文档 |
| `.specite/docs/` | 子代理写入 | 外部库调研报告 |
| `.specite/templates/` | specite 安装 | SPEC.md 和 PLAN.md 模板 |
| `.specite/iters.json` | specite 运行时 | 迭代元数据 |
