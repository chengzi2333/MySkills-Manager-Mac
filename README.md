#
<p align="center">
  <img src="./public/skillar-dark-v1-white-bg.png" alt="Skillar Logo" width="200">
</p>

<h1 align="center">Skillar (MySkills-Manager)</h1>

<p align="center">
  <strong>一个桌面应用，用于统一管理、同步和洞察您在多个 AI 编码助手（如 Antigravity, Codex, Claude Code 等）中的技能。</strong>
</p>

<p align="center">
  <a href="#-核心理念">核心理念</a> •
  <a href="#✨-主要功能">主要功能</a> •
  <a href="#-技术栈">技术栈</a> •
  <a href="#-下载">下载</a> •
  <a href="#-开发指南">开发指南</a> •
  <a href="#-项目结构">项目结构</a> •
  <a href="#-配置">配置</a>
</p>

---

**Skillar** 旨在解决在不同 AI 工具中维护和复用 `Skills` (或 `Instructions`, `Superpowers`) 的碎片化问题。它通过建立一个本地的"技能库"作为单一事实来源 (Single Source of Truth)，将您的技能高效、可靠地分发到所有支持的工具中，并提供强大的数据洞察和版本管理能力。

## 核心理念

Skillar 的设计哲学根植于三大支柱，旨在将您从繁琐的技能维护中解放出来，专注于创造和掌握与 AI 协作的最佳实践。

| 理念 | 描述 |
| :--- | :--- |
| **统一 (Unify)** | 将散落在各个工具中的技能整合到一个统一的、版本化的技能库中。一次定义，随处使用，确保一致性。 |
| **洞察 (Insight)** | 通过可观测性驱动的迭代，量化每个技能的使用频率和效果。用数据指导您优化、保留或淘汰哪些技能。 |
| **掌控 (Mastery)** | 将 AI 能力的管理权交还给您。通过直观的界面，精确控制哪些技能同步到哪些工具，实现真正的"人机协同"。 |

## ✨ 主要功能

Skillar 提供了一套完整的工作流来管理您的 AI 技能生态系统。

#### 1. **统一技能管理 (Unified Skill Management)**
- **单一事实来源**: 所有技能集中存储在您本地的 `~/my-skills` 目录中，每个技能一个文件夹，包含 `SKILL.md` 文件。
- **可视化技能列表**: 在主界面清晰地展示所有技能，包含名称、描述、分类和标签。
- **强大的技能编辑器**: 内置 Monaco Editor，提供 Markdown 语法高亮和舒适的编辑体验，支持 YAML frontmatter 元数据管理。

#### 2. **多工具同步引擎 (Multi-Tool Sync Engine)**
- **广泛的工具支持**: 内置支持 Antigravity, Codex, Claude Code, Cursor, Windsurf, Trae, OpenCode 等主流 AI 编码助手。
- **智能路径探测**: 自动检测已安装工具的技能目录，减少手动配置。
- **灵活的同步模式**: 支持 `symlink` (符号链接，推荐) 和 `copy` (文件复制) 两种模式，以适应不同工具的文件系统限制。
- **自定义工具扩展**: 允许您添加任何使用文件夹结构管理技能的自定义工具。

#### 3. **使用分析看板 (Usage Analytics Dashboard)**
- **核心 KPI**: 实时展示技能总调用次数、活跃技能数、技能总数和未使用技能数等关键指标。
- **多维度图表分析**: 
  - **高频技能**: 条形图展示最常使用的技能 Top 15。
  - **工具分布**: 饼图展示技能在不同工具上的调用分布。
  - **每日趋势**: 折线图展示每日的技能调用总次数。
- **可配置时间窗口**: 支持按最近 7 天、30 天、90 天筛选数据。

#### 4. **技能调用日志 (Skill Invocation Logs)**
- **详细的日志记录**: 记录每一次技能调用的时间戳、技能名称、工具、工作目录等信息。
- **强大的筛选与分页**: 支持按技能、工具和时间范围进行精确查询。
- **数据驱动的优化**: 通过分析日志，了解特定任务或项目中哪些技能最有效。

#### 5. **Git 集成 (Git Integration)**
- **版本化您的技能库**: 将您的 `~/my-skills` 目录作为一个 Git 仓库进行管理。
- **应用内 Git 工作流**: 在应用内即可查看文件状态（已修改、已暂存、未跟踪）、编写提交信息、执行 `commit` 和 `push` 操作。
- **保障变更安全**: 轻松追踪技能的每一次迭代，随时可以回滚到历史版本。

#### 6. **引导式设置 (Onboarding Wizard)**
- **平滑的首次启动体验**: 通过三步引导，帮助用户快速完成技能目录设置和首次工具同步。
- **一键导入**: 自动扫描所有已安装工具，一键将现有技能导入您的中央技能库。

## 技术栈

Skillar 采用现代化的技术栈，以确保高性能、可靠性和跨平台兼容性。

| 类别 | 技术 |
| :--- | :--- |
| **应用框架** | [Tauri](https://tauri.app/) (v2) - 使用 Rust 构建高性能、安全的跨平台桌面应用。 |
| **前端** | [React](https://react.dev/) (v19), [TypeScript](https://www.typescriptlang.org/), [Vite](https://vitejs.dev/) |
| **后端 (Rust)** | [Serde](https://serde.rs/) (序列化/反序列化), [git2-rs](https://github.com/rust-lang/git2-rs) (Git 操作), [rusqlite](https://github.com/rusqlite/rusqlite) (日志索引) |
| **UI & 状态管理** | 自定义组件, React Hooks |
| **图表** | [ECharts for React](https://github.com/hustcc/echarts-for-react) |
| **代码编辑器** | [Monaco Editor](https://microsoft.github.io/monaco-editor/) |

## 📥 下载

从 [GitHub Releases](https://github.com/KeithChen51/MySkills-Manager/releases/latest) 下载最新版本的 `Skillar.exe`，双击即可运行，无需安装任何开发环境。

> **系统要求**: Windows 10/11 (64-bit)

## 开发指南

如果您希望从源码构建或参与开发，请按以下步骤操作。

### 前置条件

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install) (stable)

### 快速开始

1.  **克隆仓库**
    ```bash
    git clone https://github.com/KeithChen51/MySkills-Manager.git
    cd MySkills-Manager
    ```

2.  **安装依赖**
    ```bash
    npm install
    ```

3.  **运行开发环境**
    此命令将同时启动 Vite 前端服务和 Tauri 后端。
    ```bash
    npm run dev
    ```

4.  **构建应用**
    构建生产版本的应用，可执行文件将生成在 `release/` 目录下。
    ```bash
    npm run build:desktop
    ```

### 主要 NPM 脚本

| 命令 | 说明 |
| :--- | :--- |
| `npm run dev` | 启动开发模式 |
| `npm run build` | 编译 TypeScript 和 Vite 前端 |
| `npm run build:desktop` | 完整构建 Tauri 应用并生成可执行文件 |
| `npm run lint` | 运行 ESLint 代码检查 |

## 项目结构

```
/MySkills-Manager
├── src/                # React 前端源码
│   ├── api/            # Tauri API 接口定义
│   ├── assets/         # 静态资源
│   ├── components/     # 可复用 React 组件
│   ├── pages/          # 各页面组件
│   ├── styles/         # 全局样式
│   └── main.tsx        # 应用入口
├── src-tauri/          # Rust 后端源码
│   ├── src/
│   │   ├── setup/      # 核心同步逻辑
│   │   ├── git.rs      # Git 集成
│   │   ├── skills.rs   # 技能管理
│   │   ├── stats.rs    # 统计分析
│   │   └── lib.rs      # Rust 命令定义和应用启动
│   ├── Cargo.toml      # Rust 依赖
│   └── tauri.conf.json # Tauri 配置
├── public/             # 公共静态文件
├── scripts/            # 构建脚本
└── README.md           # 本文档
```

## 配置

- **技能根目录**: Skillar 默认使用 `~/my-skills` 作为您的中央技能库。您可以通过设置 `MYSKILLS_ROOT_DIR` 环境变量来覆盖此路径。
- **应用配置**: 应用的内部配置（如初始化状态）存储在 `~/.myskills-manager/config.json`。
- **日志文件**: 技能使用日志存储在 `~/my-skills/.logs/skill-usage.jsonl`。

---


<p align="center">由 Keith Lim 构建</p>
