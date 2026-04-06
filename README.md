<div align="right">
  <strong>English</strong> | <a href="README.zh-CN.md">简体中文</a>
</div>

# Input 0

A macOS voice input tool — hold a hotkey to record, release to get polished text auto-pasted into any input field.

Local AI transcription → LLM text optimization → auto-paste. Private, fast, effortless.

<!-- [Screenshot Placeholder: Main App Interface] -->

## Features

- **Press & Speak** — Hold `Option+Space` (customizable) to record, release to transcribe + optimize + paste. No window switching needed.
- **Privacy-First Local STT** — Four AI engines (Whisper, SenseVoice, Paraformer, Moonshine) run entirely on your Mac via Metal GPU. Audio never leaves your device.
- **AI-Powered Polish** — LLM auto-corrects grammar, removes filler words, and structures your text. Built-in technical term correction (e.g. phonetic Chinese → "React"), with custom vocabulary support.
- **Auto-Paste Anywhere** — Optimized text is automatically pasted into your active input field — Slack, WeChat, VS Code, browsers, any app.
- **99+ Languages** — 4 engines, 9 models, covering 99+ languages. The system recommends the best model based on your language.
- **On-Demand Models** — Lightweight app, download only the STT models you need. One-click switch, progress display, smart recommendations.
- **ESC to Cancel** — Cancel at any stage (recording, transcribing, optimizing) by pressing ESC.
- **History** — Review past transcriptions with original and AI-optimized text side by side.
- **Custom Vocabulary** — Add professional terms, names, and product names. Auto-learning detects corrections and validates via LLM.
- **Dark & Light Themes** — Dual theme support to match your preference.
- **Liquid Glass Overlay** — Translucent recording overlay with native macOS blur, non-intrusive to your workflow.

## Supported STT Models

| Model | Size | Best For |
|-------|------|----------|
| Whisper Base | ~142 MB | Fast & lightweight, good for daily use |
| Whisper Small | ~466 MB | Balanced accuracy and speed |
| Whisper Medium | ~1.4 GB | Excellent multilingual accuracy |
| Whisper Large v3 | ~2.9 GB | Highest accuracy, 99 languages |
| Whisper Large v3 Turbo | ~1.5 GB | Top accuracy for English & multilingual |
| Whisper Large v3 Turbo Q5 | ~547 MB | Quantized high-accuracy, balanced size |
| SenseVoice Small | ~228 MB | Best for Chinese / Japanese / Korean |
| Paraformer Chinese | ~217 MB | Chinese-optimized, ultra-fast inference |
| Moonshine Base (EN) | ~274 MB | English-only, ~5x faster than Whisper |

## Tech Stack

| Layer | Technology |
|-------|------------|
| Framework | Tauri v2 (Rust + WebView) |
| Frontend | React 19 + TypeScript + Vite |
| Styling | Tailwind CSS v4 |
| Animation | Framer Motion v12 |
| State | Zustand |
| STT | whisper-rs (Metal GPU) + sherpa-onnx |
| LLM | OpenAI API compatible (GPT-4o-mini default) |
| Audio | cpal (capture) + rubato (resample) |
| Paste | arboard (clipboard) + AppleScript (Cmd+V) |
| Platform | macOS 11+ (Apple Silicon recommended) |

## System Requirements

- macOS 11.0+
- Apple Silicon processor (recommended for GPU acceleration)
- cmake (`brew install cmake`)
- Rust stable
- Node.js 20+ and pnpm

## Getting Started

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd input0
   ```

2. Install dependencies:
   ```bash
   pnpm install
   ```

3. Start development server (with hot reload):
   ```bash
   pnpm tauri dev
   ```
   The first run will prompt you to download an STT model from the Settings page.

## Build

### Production Build
```bash
MACOSX_DEPLOYMENT_TARGET=11.0 CMAKE_OSX_DEPLOYMENT_TARGET=11.0 pnpm tauri build --bundles app
```

### Run Tests
```bash
cd src-tauri && cargo test --lib
```

### Type Check
```bash
pnpm build
```

## Project Structure

```
input0/
├── src/                    # React frontend
│   ├── pages/              # Settings window, Overlay window
│   ├── stores/             # Zustand state management
│   ├── hooks/              # Tauri event hooks
│   └── components/         # UI components
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── pipeline.rs     # Voice processing pipeline state machine
│   │   ├── lib.rs          # App entry, hotkey registration, model loading
│   │   ├── audio/          # Audio capture (cpal) + format conversion (rubato)
│   │   ├── stt/            # STT backends (Whisper, SenseVoice, Paraformer, Moonshine)
│   │   ├── models/         # Model registry + download manager
│   │   ├── llm/            # LLM text optimization (GPT API)
│   │   ├── input/          # Clipboard + simulated paste
│   │   ├── config/         # TOML config read/write
│   │   ├── vocabulary.rs   # Custom vocabulary (JSON persistence)
│   │   └── commands/       # Tauri IPC commands
│   └── resources/          # STT model files
└── docs/                   # Design docs & feature specs
```

## Configuration

Config file location:
`~/Library/Application Support/com.input0.dev/config.toml`

Key settings:
- `api_key` — LLM API key
- `base_url` — LLM service endpoint
- `language` — Transcription language (auto/zh/en/ja/ko/fr/de/es/ru)
- `hotkey` — Activation hotkey

## License

This project is licensed under [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/). You are free to share and adapt, but not for commercial use.
