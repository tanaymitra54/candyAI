//! Live discovery of new open-weight speech models on Hugging Face.
//!
//! The bundled `catalog.json` is frozen at build time. This module lets the
//! Models page ask Hugging Face for GGUF speech models published since then,
//! keep only the ones transcribe-cpp can actually load (verified by reading the
//! GGUF metadata header over an HTTP Range request), and hand them to the
//! [`ModelManager`](crate::managers::model::ModelManager) as ordinary Hugging
//! Face downloads.
//!
//! Nothing here is trusted blindly: a repo is offered only when its file parses
//! as GGUF and names an architecture in
//! [`KNOWN_ARCHES`](crate::managers::model_capabilities::KNOWN_ARCHES). The pin
//! is the repo's current commit sha, so a download fetches exactly the bytes we
//! probed.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::managers::gguf_meta::{self, GgufError};
use crate::managers::model::ModelInfo;
use crate::managers::model::ModelSource;
use crate::managers::model_capabilities::{
    CapabilityProbe, Compatibility, KNOWN_ARCHES, PROBE_KEYS,
};

/// How many Hugging Face results to inspect per update.
const SEARCH_LIMIT: usize = 40;
/// Stop after this many repos pass the header check, so one press stays quick.
const MAX_NEW_MODELS: usize = 12;
/// Most repos to inspect in one press.
const MAX_CANDIDATES: usize = 24;
/// Header probes in flight at once.
const PROBE_CONCURRENCY: usize = 4;
/// Wait this long for the whole update before reporting what we have.
const UPDATE_TIMEOUT_SECS: u64 = 90;
/// Preferred quantisations, best first. Falls back to whatever root GGUF exists.
const QUANT_PREFERENCE: &[&str] = &["Q8_0", "Q6_K", "Q5_K_M", "Q4_K_M", "F16", "F32"];
/// Prefix sizes tried when reading a remote header (grows on `Truncated`).
const HEADER_PREFIXES: &[usize] = &[1 << 20, 4 << 20, 16 << 20];

const HF_API: &str = "https://huggingface.co/api";
const USER_AGENT: &str = "candyAI (model discovery)";

/// What the app remembers between launches: the models the last update found,
/// which of them the user has not seen yet, and when the check last ran.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateState {
    pub fetched_at: Option<u64>,
    /// Ids discovered but not yet shown under "New models".
    #[serde(default)]
    pub new_ids: Vec<String>,
    /// Every model discovered so far, so they survive a restart.
    #[serde(default)]
    pub models: Vec<ModelInfo>,
}

/// What the update button returns.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReport {
    /// Repos inspected.
    pub checked: usize,
    /// Ids added to the app's model list.
    pub added: Vec<String>,
    /// When the check ran (unix seconds).
    pub fetched_at: Option<u64>,
    /// Set when the network call failed; `added` is then empty.
    pub error: Option<String>,
}

/// The persisted state, as the Models page reads it.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSnapshot {
    pub fetched_at: Option<u64>,
    pub new_ids: Vec<String>,
}

/// `<app data>/model-updates.json` — beside the models directory.
pub fn state_path(app: &AppHandle) -> Option<PathBuf> {
    crate::portable::app_data_dir(app)
        .ok()
        .map(|dir| dir.join("model-updates.json"))
}

pub fn load_state(app: &AppHandle) -> UpdateState {
    let Some(path) = state_path(app) else {
        return UpdateState::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            log::warn!("Ignoring malformed {}: {}", path.display(), e);
            UpdateState::default()
        }),
        Err(_) => UpdateState::default(),
    }
}

pub fn save_state(app: &AppHandle, state: &UpdateState) {
    let Some(path) = state_path(app) else {
        return;
    };
    match serde_json::to_string_pretty(state) {
        Ok(text) => {
            if let Err(e) = std::fs::write(&path, text) {
                log::warn!("Could not save {}: {}", path.display(), e);
            }
        }
        Err(e) => log::warn!("Could not serialise model update state: {}", e),
    }
}

/* ---------------------------------- API shapes ---------------------------------- */

#[derive(Deserialize)]
struct SearchItem {
    id: String,
}

#[derive(Deserialize)]
struct RepoDetail {
    #[serde(default)]
    sha: Option<String>,
    #[serde(default)]
    siblings: Vec<Sibling>,
}

#[derive(Deserialize)]
struct Sibling {
    rfilename: String,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    lfs: Option<Lfs>,
}

#[derive(Deserialize)]
struct Lfs {
    #[serde(default)]
    size: Option<u64>,
}

impl Sibling {
    fn bytes(&self) -> Option<u64> {
        self.lfs
            .as_ref()
            .and_then(|l| l.size)
            .or(self.size)
            .filter(|n| *n > 0)
    }
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("could not start the network client: {e}"))
}

/// Quant label for a filename (`model-Q8_0.gguf` → `Q8_0`), else the stem.
fn quant_of(filename: &str) -> String {
    let lower = filename.to_ascii_uppercase();
    for quant in QUANT_PREFERENCE {
        if lower.contains(&format!("-{quant}.")) || lower.contains(&format!("_{quant}.")) {
            return (*quant).to_string();
        }
    }
    "GGUF".to_string()
}

fn rank_of(filename: &str) -> usize {
    let quant = quant_of(filename);
    QUANT_PREFERENCE
        .iter()
        .position(|q| *q == quant)
        .unwrap_or(QUANT_PREFERENCE.len())
}

/// The repo's root-level GGUFs (subfolders can't be fetched by name alone),
/// best quant first.
fn candidates(detail: &RepoDetail) -> Vec<&Sibling> {
    let mut files: Vec<&Sibling> = detail
        .siblings
        .iter()
        .filter(|s| {
            s.rfilename.to_ascii_lowercase().ends_with(".gguf") && !s.rfilename.contains('/')
        })
        .collect();
    files.sort_by_key(|s| rank_of(&s.rfilename));
    files
}

/// Read the GGUF metadata header of a remote file without downloading it.
async fn probe_remote(
    client: &reqwest::Client,
    repo_id: &str,
    revision: &str,
    filename: &str,
) -> Option<CapabilityProbe> {
    let url = format!("https://huggingface.co/{repo_id}/resolve/{revision}/{filename}");
    for size in HEADER_PREFIXES {
        let response = client
            .get(&url)
            .header(reqwest::header::RANGE, format!("bytes=0-{}", size - 1))
            .send()
            .await
            .ok()?;
        if !response.status().is_success() {
            return None;
        }
        // Read at most `size` bytes even if the host ignores Range and starts
        // sending the whole file.
        let mut bytes = Vec::with_capacity(*size);
        let mut stream = response;
        while bytes.len() < *size {
            match stream.chunk().await {
                Ok(Some(chunk)) => {
                    let take = chunk.len().min(size - bytes.len());
                    bytes.extend_from_slice(&chunk[..take]);
                }
                Ok(None) => break,
                Err(_) => return None,
            }
        }
        match gguf_meta::parse_header(&bytes, PROBE_KEYS) {
            Ok(meta) => return Some(CapabilityProbe::from_metadata(&meta)),
            Err(GgufError::Truncated { .. }) => continue,
            Err(_) => return None,
        }
    }
    None
}

async fn build_model(
    client: &reqwest::Client,
    repo_id: &str,
    revision: &str,
    file: &Sibling,
) -> Option<ModelInfo> {
    let probe = probe_remote(client, repo_id, revision, &file.rfilename).await?;
    // Only models this app's engine can load.
    let architecture = probe.architecture.clone()?;
    if probe.verdict != Compatibility::Compatible || !KNOWN_ARCHES.contains(&architecture.as_str())
    {
        return None;
    }
    let filename = &file.rfilename;
    let name = probe
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            filename
                .trim_end_matches(".gguf")
                .trim_end_matches(".GGUF")
                .to_string()
        });
    let languages = probe.languages.clone().unwrap_or_default();
    Some(ModelInfo {
        id: format!("{repo_id}/{filename}"),
        name,
        description: format!("From Hugging Face ({repo_id}). Not tested with candyAI."),
        filename: filename.clone(),
        source: ModelSource::HuggingFace {
            repo_id: repo_id.to_string(),
            revision: revision.to_string(),
        },
        size_mb: file.bytes().unwrap_or(0) / (1024 * 1024),
        is_downloaded: false,
        is_downloading: false,
        partial_size: 0,
        // Unmeasured: the UI shows "?" rather than a score.
        accuracy_score: 0.0,
        speed_score: 0.0,
        supports_translation: probe.supports_translation.unwrap_or(false),
        is_recommended: false,
        supports_language_selection: languages.len() > 1,
        supported_languages: languages,
        is_custom: false,
        supports_streaming: probe.supports_streaming.unwrap_or(false),
        supports_language_detection: probe.supports_language_detect.unwrap_or(false),
    })
}

/// Repos already represented in the app's model list.
pub fn known_repos(models: impl Iterator<Item = ModelInfo>) -> HashSet<String> {
    models
        .filter_map(|m| match m.source {
            ModelSource::HuggingFace { repo_id, .. } => Some(repo_id),
            _ => None,
        })
        .collect()
}

/// One Hugging Face search, newest first. Errors are per-query: one broken
/// query shouldn't sink the whole check.
async fn search(client: &reqwest::Client, query: &str) -> Result<Vec<String>, String> {
    let url =
        format!("{HF_API}/models?{query}&sort=lastModified&direction=-1&limit={SEARCH_LIMIT}");
    let items: Vec<SearchItem> = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Hugging Face is unreachable: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Hugging Face returned an error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Hugging Face sent something unreadable: {e}"))?;
    Ok(items.into_iter().map(|i| i.id).collect())
}

/// Ask Hugging Face for speech GGUFs, verify each header, and return the ones
/// the app can use. Network failure is an error; an empty list is not.
pub async fn discover(known: &HashSet<String>) -> Result<(Vec<ModelInfo>, usize), String> {
    let client = client()?;

    // Two different nets: repos built for transcribe.cpp (tagged), and speech
    // GGUFs in general. The header check below is what actually admits a model,
    // so casting wider costs only a few requests.
    let tagged = search(&client, "filter=transcribe.cpp").await;
    let speech = search(
        &client,
        "filter=gguf&pipeline_tag=automatic-speech-recognition",
    )
    .await;
    let mut ids: Vec<String> = Vec::new();
    for query in [tagged, speech] {
        let Ok(found) = query else { continue };
        for id in found {
            if !known.contains(&id) && !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    if ids.is_empty() {
        return Ok((Vec::new(), 0));
    }
    // Bound the work: each candidate costs one repo lookup and one header read.
    ids.truncate(MAX_CANDIDATES);
    let checked = ids.len();

    let work = futures_util::stream::iter(ids.into_iter().map(|repo_id| {
        let client = client.clone();
        async move {
            let detail: RepoDetail = client
                .get(format!("{HF_API}/models/{repo_id}?blobs=true"))
                .send()
                .await
                .ok()?
                .error_for_status()
                .ok()?
                .json()
                .await
                .ok()?;
            let revision = detail.sha.clone()?;
            let file = *candidates(&detail).first()?;
            build_model(&client, &repo_id, &revision, file).await
        }
    }))
    .buffer_unordered(PROBE_CONCURRENCY)
    .filter_map(|m| async move { m });

    let found = tokio::time::timeout(
        std::time::Duration::from_secs(UPDATE_TIMEOUT_SECS),
        work.collect::<Vec<ModelInfo>>(),
    )
    .await
    .unwrap_or_default()
    .into_iter()
    .take(MAX_NEW_MODELS)
    .collect();

    Ok((found, checked))
}

/// Unix seconds, for the state file.
pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sibling(name: &str, size: Option<u64>) -> Sibling {
        Sibling {
            rfilename: name.to_string(),
            size,
            lfs: None,
        }
    }

    #[test]
    fn quant_preference_picks_q8_then_next() {
        let detail = RepoDetail {
            sha: Some("abc".into()),
            siblings: vec![
                sibling("m-Q4_K_M.gguf", Some(4)),
                sibling("m-Q8_0.gguf", Some(8)),
                sibling("README.md", None),
                sibling("sub/m-Q8_0.gguf", Some(8)),
            ],
        };
        let files = candidates(&detail);
        assert_eq!(files.len(), 2, "subfolder files are not offered");
        assert_eq!(files[0].rfilename, "m-Q8_0.gguf");
    }

    /// Real network: `cargo test --lib -- --ignored --nocapture`.
    #[tokio::test]
    #[ignore = "hits the Hugging Face API"]
    async fn discovers_loadable_models() {
        let (found, checked) = discover(&HashSet::new()).await.expect("network");
        println!("checked {checked}, found {}", found.len());
        for m in &found {
            println!("{} — {} ({} languages)", m.name, m.id, m.supported_languages.len());
        }
    }

    #[test]
    fn quant_label_reads_the_filename() {
        assert_eq!(quant_of("model-Q8_0.gguf"), "Q8_0");
        assert_eq!(quant_of("model-q4_k_m.GGUF"), "Q4_K_M");
        assert_eq!(quant_of("model.gguf"), "GGUF");
    }
}
