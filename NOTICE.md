# NOTICE

candyAI is a redistribution of Handy.

- Upstream project: https://github.com/cjpais/Handy
- Upstream copyright: Copyright (c) 2025 CJ Pais
- Upstream licence: MIT, reproduced in full in [LICENSE](LICENSE)

The MIT licence permits use, modification, and redistribution provided the copyright notice and
permission notice are retained. Both are present in this repository.

## Modifications in this distribution

- Product name, bundle identifier (`com.candyai.io`), window title, tray label, About panel.
- Application icon and in-app wordmark replaced; mascot recoloured.
- Complete UI palette replaced: dark only, warm orange. Light mode removed.
- Contrast fixes on primary buttons, badges, radio controls, and the update bar.
- Product name updated across all 26 locale files.
- Default transcription model set to `parakeet-unified-en-0.6b` (Q8_0).
- Self-update disabled: no update endpoints, no updater artifacts.
- Upstream donation prompt removed from About.

## Third-party components

- [transcribe-cpp](https://crates.io/crates/transcribe-cpp) and ggml/llama.cpp — local inference
- [tauri](https://tauri.app) — application shell
- Parakeet Unified English 0.6B model weights — NVIDIA, CC-BY-4.0. Downloaded at runtime, not
  distributed in this repository.
- Silero VAD — see upstream model card for terms.

Nothing in this repository is endorsed by the upstream authors.
