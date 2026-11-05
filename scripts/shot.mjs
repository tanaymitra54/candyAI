import { chromium } from "playwright";
import fs from "fs";
import os from "os";
import path from "path";

/**
 * Renders candyAI's real frontend in Chromium with a stubbed Tauri IPC layer.
 * Tauri's bindings wrap every command result as { status, data }, so the stubs
 * below return the RAW payload the Rust command would have produced.
 */
const storePath = `${os.homedir()}/Library/Application Support/com.candyai.io/settings_store.json`;
const settings = JSON.parse(fs.readFileSync(storePath, "utf8")).settings;

const MODELS = [
  {
    id: "handy-computer/parakeet-unified-en-0.6b-gguf/parakeet-unified-en-0.6b-Q8_0.gguf",
    name: "Parakeet Unified English 0.6B",
    description: "Best all-around choice for English dictation",
    filename: "parakeet-unified-en-0.6b-Q8_0.gguf",
    source: { HuggingFace: { repo_id: "handy-computer/parakeet-unified-en-0.6b-gguf", revision: "7e948f21b7bdbac698d3318db9d350f1096f3b6c" } },
    size_mb: 731,
    is_downloaded: true,
    is_downloading: false,
    partial_size: 731,
    accuracy_score: 5,
    speed_score: 5,
    supports_translation: false,
    is_recommended: true,
    supported_languages: ["en"],
    supports_language_selection: false,
    is_custom: false,
    supports_streaming: true,
    supports_language_detection: false,
  },
  {
    id: "handy-computer/whisper-large-v3-turbo-gguf/whisper-large-v3-turbo-Q5_0.gguf",
    name: "Whisper Large v3 Turbo",
    description: "Wide language coverage, slower on older machines",
    filename: "whisper-large-v3-turbo-Q5_0.gguf",
    source: { HuggingFace: { repo_id: "handy-computer/whisper-large-v3-turbo-gguf", revision: "main" } },
    size_mb: 809,
    is_downloaded: false,
    is_downloading: false,
    partial_size: 0,
    accuracy_score: 5,
    speed_score: 3,
    supports_translation: true,
    is_recommended: false,
    supported_languages: ["en", "hi", "ta"],
    supports_language_selection: true,
    is_custom: false,
    supports_streaming: false,
    supports_language_detection: true,
  },
];

const HISTORY = {
  entries: [
    {
      id: 3,
      file_name: "candy-1730812345.wav",
      timestamp: 1730812345,
      saved: false,
      transcription_text:
        "Ship the dark orange theme tonight, then review the model list tomorrow morning.",
      post_processed_text: null,
      post_process_prompt: null,
      post_process_requested: false,
      source_name: "Dictation",
      language: "en",
      language_source: "selected",
    },
    {
      id: 2,
      file_name: "candy-1730812001.wav",
      timestamp: 1730812001,
      saved: true,
      transcription_text:
        "Reminder: re-run the benchmark on the MacBook before changing the quantization.",
      post_processed_text: null,
      post_process_prompt: null,
      post_process_requested: false,
      source_name: "Dictation",
      language: "en",
      language_source: "selected",
    },
  ],
  next_cursor: null,
};

const RAW = {
  get_app_settings: settings,
  get_default_settings: settings,
  get_settings: settings,
  get_available_models: MODELS,
  get_current_model: settings.selected_model,
  get_transcription_model_status: { is_loaded: true, current_model: settings.selected_model },
  get_model_load_status: { is_loaded: true, current_model: settings.selected_model },
  get_model_info: MODELS,
  get_history_entries: HISTORY,
  get_active_notices: [],
  get_retired_models: [],
  get_microphone_mode: "always_on",
  get_selected_microphone: "MacBook Pro Microphone",
  get_selected_output_devices: [],
  get_selected_microphone_channels: [],
  list_devices: { devices: [], loaded: false },
  get_audio_file_path: null,
  get_secure_input_status: {
    enabled: false,
    sustained: false,
    culprit_pid: null,
    culprit_name: null,
  },
  get_available_accelerators: ["metal"],
  get_gpu_devices: [],
  is_language_detector_downloaded: true,
  get_language_status: { status: "ok" },
  is_recording: false,
  check_accessibility_permission: true,
  check_microphone_permission: true,
  check_input_monitoring_permission: true,
  check_screen_recording_permission: true,
  check_full_disk_access_permission: true,
  check_custom_sounds: { start: true, stop: true, unavailable: [] },
  is_update_checks_locked: false,
  is_update_check_pending: false,
  get_app_dir: `${os.homedir()}/Library/Application Support/com.candyai.io`,
  get_app_dir_path: `${os.homedir()}/Library/Application Support/com.candyai.io`,
  get_log_dir_path: `${os.homedir()}/Library/Logs/com.candyai.io`,
  get_microphone_channels: [],
  "plugin:app|version": "1.0.0",
  "plugin:app|name": "candyAI",
  "plugin:app|tauri_version": "2.11.5",
  get_log_dir: `${os.homedir()}/Library/Logs/com.candyai.io`,
  "plugin:event|listen": 0,
  "plugin:event|unlisten": null,
  "plugin:macos-permissions|check_accessibility_permission": true,
  "plugin:macos-permissions|check_microphone_permission": true,
  "plugin:macos-permissions|check_input_monitoring_permission": true,
  "plugin:macos-permissions|check_full_disk_access_permission": true,
  "plugin:macos-permissions|check_screen_recording_permission": true,
  "show_main_window_command": null,
  "plugin:os|locale": "en-US",
  "plugin:os|platform": "macos",
  "plugin:os|os_type": "macos",
  "plugin:os|arch": "aarch64",
  "plugin:os|version": "15.0.0",
  "plugin:os|family": "unix",
  "plugin:os|exe_extension": "",
  "plugin:os|hostname": "mac",
};

const shots = [
  { name: "general", page: "General" },
  { name: "models", page: "Models" },
  { name: "history", page: "History" },
  { name: "advanced", page: "Advanced" },
  { name: "about", page: "About" },
];

const browser = await chromium.launch();
const page = await browser.newPage({
  viewport: { width: 1100, height: 720 },
  deviceScaleFactor: 2,
});
let errShown=false;
page.on("pageerror", (e) => { if(!errShown){errShown=true; console.log("PAGEERROR:", e.message.slice(0,160), "\nSTACK:", (e.stack||"").split("\n").slice(0,5).join(" || "));} });
page.on("console", (m) => { const t = m.text(); if (t.startsWith("UNKNOWN:")) console.log(t); });

await page.addInitScript((raw) => {
  window.__TAURI_INTERNALS__ = {
    invoke: (cmd) => {
      if (cmd in raw) return Promise.resolve(raw[cmd]);
      console.log("UNKNOWN:" + cmd);
      return Promise.resolve(null);
    },
    transformCallback: (cb) => {
      const id = Math.floor(Math.random() * 1e6);
      window[`_${id}`] = cb;
      return id;
    },
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    convertFileSrc: (p) => p,
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
  window.__TAURI_OS_PLUGIN_INTERNALS__ = {
    os_type: "macos",
    arch: "aarch64",
    platform: "macos",
    version: "15.0.0",
    family: "unix",
    exe_extension: "",
    locale: "en-US",
  };
}, RAW);

await page.goto("http://localhost:1420/", { waitUntil: "networkidle" });
await page.waitForTimeout(2500);

const outDir = path.resolve("docs/screenshots");
fs.mkdirSync(outDir, { recursive: true });

for (const s of shots) {
  const item = page.getByText(s.page, { exact: true }).first();
  if (await item.count()) {
    await item.click({ timeout: 3000 }).catch(() => {});
    await page.waitForTimeout(700);
    await page.screenshot({ path: path.join(outDir, `${s.name}.png`) });
    console.log("captured", s.name);
  } else {
    console.log("no nav item", s.page);
  }
}

console.log("done");
await browser.close();
