# candyAI

A private, on-device speech-to-text app for macOS. Hold a key, speak, and the text is typed
for you. Audio never leaves the machine.

candyAI is a redistribution of [Handy](https://github.com/cjpais/Handy) by CJ Pais, rebranded
and restyled. See [NOTICE.md](NOTICE.md) for the attribution this licence requires.

## What it does

- Global shortcut dictation, push-to-talk or toggle
- Local transcription on Metal, no network calls
- Default model: **Parakeet Unified English 0.6B** (Q8_0, GGUF)
- Recording overlay, tray control, transcription history
- Optional post-processing through an LLM provider you configure yourself

## Design

Dark only, no light mode. Warm orange palette, one token file:

| Role | Token | Value |
|---|---|---|
| Background | `--color-background` | `#140e09` |
| Text | `--color-text` | `#fdf1e3` |
| Primary | `--color-logo-primary` | `#ff8a3d` |
| Action fill | `--color-background-ui` | `#ff7a1a` |
| Accent | `--color-orange-soft` | `#ffc79a` |

All colours live in `src/styles/theme.css`. Contrast was audited to WCAG 2.2; every body-text
pair clears 4.5:1.

## Requirements

- macOS (arm64 build verified on Apple Silicon)
- Rust stable
- [Bun](https://bun.sh)
- Xcode Command Line Tools

## Build

```bash
bun install

mkdir -p src-tauri/resources/models
curl -L -o src-tauri/resources/models/silero-vad-v6.2-F32.gguf \
  https://huggingface.co/handy-computer/silero-vad-v6.2-gguf/resolve/main/silero-vad-v6.2-F32.gguf

CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri build --bundles app
```

The bundle lands in `src-tauri/target/release/bundle/macos/candyAI.app`.

No notarization is configured. The build is ad-hoc signed and runs only on the machine that
produced it unless you supply a Developer ID.

## Model

The default model is set in `src-tauri/src/settings.rs`:

```
handy-computer/parakeet-unified-en-0.6b-gguf/parakeet-unified-en-0.6b-Q8_0.gguf
```

Weights are downloaded on first use from Hugging Face and cached in
`~/Library/Application Support/com.candyai.io/models/`. They are not in this repo.

## Updates

Self-update is disabled. `plugins.updater.endpoints` is empty and no updater artifacts are
produced. Install new builds manually.

## Licence

MIT. The upstream copyright notice is retained in [LICENSE](LICENSE) and the modifications are
listed in [NOTICE.md](NOTICE.md). The default model weights are licensed **CC-BY-4.0** by NVIDIA,
separately from the application code.
