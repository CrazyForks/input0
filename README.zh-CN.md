<div align="right">
  <a href="README.md">English</a> | <strong>简体中文</strong>
</div>

# Input 0

macOS 语音输入工具 — 按住快捷键录音，松开后自动转写、优化、粘贴到当前输入框。

本地 AI 转录 → LLM 文本优化 → 自动粘贴。隐私、快速、零摩擦。

<!-- [Screenshot Placeholder: Main App Interface] -->

## 功能特性

- **一键语音输入** — 按住 `Option+Space`（可自定义）录音，松开即自动转写 + 优化 + 粘贴，无需切换窗口。
- **隐私优先的本地转录** — 四大 AI 引擎（Whisper、SenseVoice、Paraformer、Moonshine）通过 Metal GPU 完全在你的 Mac 上本地运行，音频数据永不离开设备。
- **AI 文本优化** — LLM 自动修正语病、去除口吃和语气词、结构化表达。内置 40+ 技术术语拼音纠错（如"瑞嗯特" → "React"），支持自定义词汇库。
- **自动粘贴** — 优化后的文本自动粘贴到当前聚焦的输入框 — Slack、微信、VS Code、浏览器，任何 App。
- **99+ 语言支持** — 4 大引擎、9 个模型，覆盖 99+ 种语言，系统根据你的语言自动推荐最佳模型。
- **模型按需管理** — 应用体积轻量，STT 模型按需下载。一键切换、进度显示、智能推荐。
- **ESC 取消** — 录音、转写、优化过程中随时按 ESC 取消，不会有任何文字被粘贴。
- **历史记录** — 查看最近的语音转录原文与 AI 优化结果，支持对比和一键复制。
- **自定义词汇库** — 手动添加专业术语、人名、产品名，AI 转录优化时优先使用。支持自动学习，LLM 验证后自动加入词汇库。
- **暗黑 / 亮色主题** — 双主题支持，匹配你的使用偏好。
- **液态玻璃 Overlay** — 录音时屏幕底部出现半透明浮层，macOS 原生毛玻璃效果，不遮挡工作区。

## 支持的 STT 模型

| 模型 | 大小 | 最佳场景 |
|------|------|---------|
| Whisper Base | ~142 MB | 快速轻量，适合日常使用 |
| Whisper Small | ~466 MB | 性价比高，精度与速度均衡 |
| Whisper Medium | ~1.4 GB | 多语言转录精度优秀 |
| Whisper Large v3 | ~2.9 GB | 最高精度，99 种语言 |
| Whisper Large v3 Turbo | ~1.5 GB | 英文/多语言最高精度 |
| Whisper Large v3 Turbo Q5 | ~547 MB | 高精度量化版，平衡大小与质量 |
| SenseVoice Small | ~228 MB | 中文/日文/韩文识别最佳 |
| Paraformer 中文 | ~217 MB | 中文专用，推理极快 |
| Moonshine Base (EN) | ~274 MB | 英文专用，速度约为 Whisper 的 5 倍 |

## 技术栈

| 层级 | 技术 |
|------|------|
| 框架 | Tauri v2 (Rust + WebView) |
| 前端 | React 19 + TypeScript + Vite |
| 样式 | Tailwind CSS v4 |
| 动画 | Framer Motion v12 |
| 状态管理 | Zustand |
| STT 引擎 | whisper-rs (Metal GPU) + sherpa-onnx |
| LLM | OpenAI API 兼容（默认 GPT-4o-mini） |
| 音频 | cpal（采集） + rubato（重采样） |
| 粘贴 | arboard（剪贴板） + AppleScript (Cmd+V) |
| 平台 | macOS 11+（推荐 Apple Silicon） |

## 系统要求

- macOS 11.0+
- Apple Silicon 处理器（推荐，以获得最佳 GPU 加速效果）
- cmake（`brew install cmake`）
- Rust 稳定版
- Node.js 20+ 及 pnpm

## 快速开始

1. 克隆项目仓库：
   ```bash
   git clone <repository-url>
   cd input0
   ```

2. 安装依赖：
   ```bash
   pnpm install
   ```

3. 启动开发服务器（支持热更新）：
   ```bash
   pnpm tauri dev
   ```
   首次运行需在设置页面下载 STT 模型。

## 构建

### 生产构建
```bash
MACOSX_DEPLOYMENT_TARGET=11.0 CMAKE_OSX_DEPLOYMENT_TARGET=11.0 pnpm tauri build --bundles app
```

### 运行测试
```bash
cd src-tauri && cargo test --lib
```

### 类型检查
```bash
pnpm build
```

## 项目结构

```
input0/
├── src/                    # React 前端
│   ├── pages/              # 设置窗口、Overlay 浮层
│   ├── stores/             # Zustand 状态管理
│   ├── hooks/              # Tauri 事件钩子
│   └── components/         # UI 组件
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── pipeline.rs     # 语音处理流水线状态机
│   │   ├── lib.rs          # 应用入口、快捷键注册、模型加载
│   │   ├── audio/          # 音频采集 (cpal) + 格式转换 (rubato)
│   │   ├── stt/            # STT 后端（Whisper、SenseVoice、Paraformer、Moonshine）
│   │   ├── models/         # 模型注册表 + 下载管理
│   │   ├── llm/            # LLM 文本优化（GPT API）
│   │   ├── input/          # 剪贴板操作 + 模拟粘贴
│   │   ├── config/         # TOML 配置文件读写
│   │   ├── vocabulary.rs   # 自定义词汇库（JSON 持久化）
│   │   └── commands/       # Tauri IPC 命令
│   └── resources/          # STT 模型文件
└── docs/                   # 设计文档与需求说明
```

## 配置说明

配置文件路径：
`~/Library/Application Support/com.input0.dev/config.toml`

主要配置项：
- `api_key` — LLM API 密钥
- `base_url` — LLM 服务地址
- `language` — 转写语言（auto/zh/en/ja/ko/fr/de/es/ru）
- `hotkey` — 唤起快捷键

## 许可证

本项目采用 [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/) 许可证。你可以自由分享和修改，但不得用于商业用途。
