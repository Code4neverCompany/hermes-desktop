use dirs::home_dir;
use regex::Regex;
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

const MEMORY_ENTRY_DELIMITER: &str = "\n§\n";
const MEMORY_CHAR_LIMIT: usize = 2200;
const USER_CHAR_LIMIT: usize = 1375;
const NO_KEY_REQUIRED: &str = "no-key-required";
const DEFAULT_SOUL: &str = r#"You are Hermes, a helpful AI assistant. You are friendly, knowledgeable, and always eager to help.

You communicate clearly and concisely. When asked to perform tasks, you think step-by-step and explain your reasoning. You are honest about your limitations and ask for clarification when needed.

You strive to be helpful while being safe and responsible. You respect the user's privacy and handle sensitive information carefully.
"#;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallStatus {
    installed: bool,
    configured: bool,
    has_api_key: bool,
    verified: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelConfig {
    provider: String,
    model: String,
    base_url: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelRecord {
    id: String,
    name: String,
    provider: String,
    model: String,
    base_url: String,
    created_at: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Claw3dStatus {
    cloned: bool,
    installed: bool,
    dev_server_running: bool,
    adapter_running: bool,
    port: u16,
    port_in_use: bool,
    ws_url: String,
    running: bool,
    error: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionResult {
    success: bool,
    error: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileInfo {
    name: String,
    path: String,
    is_default: bool,
    is_active: bool,
    model: String,
    provider: String,
    has_env: bool,
    has_soul: bool,
    skill_count: usize,
    gateway_running: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MemoryEntry {
    index: usize,
    content: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MemoryFileState {
    content: String,
    exists: bool,
    last_modified: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MemoryDetail {
    content: String,
    exists: bool,
    last_modified: Option<u64>,
    entries: Vec<MemoryEntry>,
    char_count: usize,
    char_limit: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserDetail {
    content: String,
    exists: bool,
    last_modified: Option<u64>,
    char_count: usize,
    char_limit: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionStats {
    total_sessions: i64,
    total_messages: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MemoryState {
    memory: MemoryDetail,
    user: UserDetail,
    stats: SessionStats,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionSummary {
    id: String,
    source: String,
    started_at: i64,
    ended_at: Option<i64>,
    message_count: i64,
    model: String,
    title: Option<String>,
    preview: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CachedSession {
    id: String,
    title: String,
    started_at: i64,
    source: String,
    message_count: i64,
    model: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionMessage {
    id: i64,
    role: String,
    content: String,
    timestamp: i64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    session_id: String,
    title: Option<String>,
    started_at: i64,
    source: String,
    message_count: i64,
    model: String,
    snippet: String,
}

#[derive(Default)]
struct ChatState {
    pid: Mutex<Option<u32>>,
    aborted: AtomicBool,
}

#[derive(Default)]
struct GatewayState {
    pid: Mutex<Option<u32>>,
}

fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn hermes_home() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hermes")
}

fn profile_home(profile: Option<&str>) -> PathBuf {
    match profile {
        Some(name) if !name.is_empty() && name != "default" => {
            hermes_home().join("profiles").join(name)
        }
        _ => hermes_home(),
    }
}

fn env_path(profile: Option<&str>) -> PathBuf {
    profile_home(profile).join(".env")
}

fn config_path(profile: Option<&str>) -> PathBuf {
    profile_home(profile).join("config.yaml")
}

fn soul_path(profile: Option<&str>) -> PathBuf {
    profile_home(profile).join("SOUL.md")
}

fn memory_path(profile: Option<&str>) -> PathBuf {
    profile_home(profile).join("MEMORY.md")
}

fn user_path(profile: Option<&str>) -> PathBuf {
    profile_home(profile).join("USER.md")
}

fn state_db_path(profile: Option<&str>) -> PathBuf {
    profile_home(profile).join("state.db")
}

fn models_path() -> PathBuf {
    hermes_home().join("models.json")
}

fn hermes_repo() -> PathBuf {
    hermes_home().join("hermes-agent")
}

fn hermes_python() -> PathBuf {
    hermes_repo().join("venv").join("bin").join("python")
}

fn hermes_script() -> PathBuf {
    hermes_repo().join("hermes")
}

fn get_enhanced_path() -> String {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let extras = [
        home.join(".local/bin"),
        home.join(".cargo/bin"),
        home.join(".volta/bin"),
        home.join(".asdf/shims"),
        hermes_repo().join("venv/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/opt/homebrew/sbin"),
    ];

    let mut joined = extras
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    joined.push(std::env::var("PATH").unwrap_or_default());
    joined.join(":")
}

fn read_file(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn safe_write_file(path: &Path, content: &str) -> Result<(), String> {
    ensure_parent_dir(path)?;
    fs::write(path, content).map_err(|err| err.to_string())
}

fn read_env_map(profile: Option<&str>) -> Vec<(String, String)> {
    read_file(&env_path(profile))
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || !trimmed.contains('=') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            Some((
                key.trim().to_string(),
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string(),
            ))
        })
        .collect()
}

fn write_env_value(key: &str, value: &str, profile: Option<&str>) -> Result<bool, String> {
    let path = env_path(profile);
    let line_value = format!("{key}={value}");

    if !path.exists() {
        safe_write_file(&path, &(line_value + "\n"))?;
        return Ok(true);
    }

    let content = read_file(&path).unwrap_or_default();
    let re = Regex::new(&format!(r"(?m)^#?\s*{}\s*=.*$", regex::escape(key)))
        .map_err(|err| err.to_string())?;

    let updated = if re.is_match(&content) {
        re.replace(&content, line_value.as_str()).to_string()
    } else if content.trim().is_empty() {
        line_value.clone()
    } else {
        format!("{}\n{}", content.trim_end_matches('\n'), line_value)
    };

    safe_write_file(&path, &(updated + "\n"))?;
    Ok(true)
}

fn read_config_value(key: &str, profile: Option<&str>) -> Option<String> {
    let content = read_file(&config_path(profile))?;
    let pattern = format!(r#"(?m)^\s*{}:\s*["']?([^"'#\n]+)"?"#, regex::escape(key));
    let re = Regex::new(&pattern).ok()?;
    re.captures(&content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
}

fn replace_config_value(key: &str, value: &str, profile: Option<&str>) -> Result<bool, String> {
    let path = config_path(profile);
    let Some(content) = read_file(&path) else {
        return Ok(false);
    };

    let pattern = format!(
        r#"(?m)^(\s*#?\s*{}:\s*)["']?[^"'#\n]*["']?"#,
        regex::escape(key)
    );
    let re = Regex::new(&pattern).map_err(|err| err.to_string())?;
    if !re.is_match(&content) {
        return Ok(false);
    }

    let updated = re.replace(&content, format!("$1\"{}\"", value)).to_string();
    safe_write_file(&path, &updated)?;
    Ok(true)
}

fn load_model_config(profile: Option<&str>) -> ModelConfig {
    ModelConfig {
        provider: read_config_value("provider", profile).unwrap_or_else(|| "auto".to_string()),
        model: read_config_value("default", profile).unwrap_or_default(),
        base_url: read_config_value("base_url", profile).unwrap_or_default(),
    }
}

fn default_models() -> Vec<ModelRecord> {
    vec![
        ModelRecord {
            id: "default-openrouter-sonnet-4".into(),
            name: "Claude Sonnet 4".into(),
            provider: "openrouter".into(),
            model: "anthropic/claude-sonnet-4-20250514".into(),
            base_url: "".into(),
            created_at: 0,
        },
        ModelRecord {
            id: "default-anthropic-sonnet-4".into(),
            name: "Claude Sonnet 4".into(),
            provider: "anthropic".into(),
            model: "claude-sonnet-4-20250514".into(),
            base_url: "".into(),
            created_at: 0,
        },
        ModelRecord {
            id: "default-openai-gpt-4-1".into(),
            name: "GPT-4.1".into(),
            provider: "openai".into(),
            model: "gpt-4.1".into(),
            base_url: "".into(),
            created_at: 0,
        },
    ]
}

fn open_db_readonly(profile: Option<&str>) -> Option<Connection> {
    let path = state_db_path(profile);
    if !path.exists() {
        return None;
    }
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()
}

fn open_db_readwrite(profile: Option<&str>) -> Option<Connection> {
    let path = state_db_path(profile);
    if !path.exists() {
        return None;
    }
    Connection::open(path).ok()
}

fn read_file_state(path: &Path) -> MemoryFileState {
    match fs::read_to_string(path) {
        Ok(content) => {
            let last_modified = fs::metadata(path)
                .ok()
                .and_then(|meta| meta.modified().ok())
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs());
            MemoryFileState {
                content,
                exists: true,
                last_modified,
            }
        }
        Err(_) => MemoryFileState {
            content: String::new(),
            exists: false,
            last_modified: None,
        },
    }
}

fn parse_memory_entries(content: &str) -> Vec<MemoryEntry> {
    if content.trim().is_empty() {
        return vec![];
    }
    content
        .split(MEMORY_ENTRY_DELIMITER)
        .enumerate()
        .filter_map(|(index, entry)| {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(MemoryEntry {
                    index,
                    content: trimmed.to_string(),
                })
            }
        })
        .collect()
}

fn serialize_memory_entries(entries: &[MemoryEntry]) -> String {
    entries
        .iter()
        .map(|entry| entry.content.clone())
        .collect::<Vec<_>>()
        .join(MEMORY_ENTRY_DELIMITER)
}

fn get_session_stats(profile: Option<&str>) -> SessionStats {
    let Some(conn) = open_db_readonly(profile) else {
        return SessionStats {
            total_sessions: 0,
            total_messages: 0,
        };
    };

    let total_sessions = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .unwrap_or(0);
    let total_messages = conn
        .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
        .unwrap_or(0);

    SessionStats {
        total_sessions,
        total_messages,
    }
}

fn get_active_profile_name() -> String {
    read_file(&hermes_home().join("active_profile"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "default".to_string())
}

fn read_profile_config(profile_path: &Path) -> (String, String) {
    let config = read_file(&profile_path.join("config.yaml")).unwrap_or_default();
    let model = Regex::new(r#"(?m)^\s*default:\s*["']?([^"'\n#]+)["']?"#)
        .ok()
        .and_then(|re| re.captures(&config))
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();
    let provider = Regex::new(r#"(?m)^\s*provider:\s*["']?([^"'\n#]+)["']?"#)
        .ok()
        .and_then(|re| re.captures(&config))
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_else(|| "auto".to_string());

    (model, provider)
}

fn count_skills(profile_path: &Path) -> usize {
    let skills_root = profile_path.join("skills");
    let Ok(categories) = fs::read_dir(skills_root) else {
        return 0;
    };

    let mut count = 0;
    for category in categories.flatten() {
        let category_path = category.path();
        if !category_path.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(category_path) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("SKILL.md").exists() {
                count += 1;
            }
        }
    }
    count
}

fn is_gateway_running(profile_path: &Path) -> bool {
    let pid_file = profile_path.join("gateway.pid");
    let Some(raw) = read_file(&pid_file) else {
        return false;
    };
    let pid = raw.trim();
    if pid.is_empty() {
        return false;
    }
    Command::new("kill")
        .arg("-0")
        .arg(pid)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn hermes_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new(hermes_python())
        .arg(hermes_script())
        .args(args)
        .current_dir(hermes_repo())
        .env("PATH", get_enhanced_path())
        .env("HERMES_HOME", hermes_home())
        .env("HOME", home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .output()
        .map_err(|err| err.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err("Hermes command failed".to_string())
        } else {
            Err(stderr)
        }
    }
}

fn is_process_running(pid: u32) -> bool {
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn kill_process(pid: u32, signal: &str) {
    let _ = Command::new("kill")
        .arg(signal)
        .arg(pid.to_string())
        .status();
}

fn read_gateway_pid_file() -> Option<u32> {
    let pid_file = hermes_home().join("gateway.pid");
    let raw = read_file(&pid_file)?;
    let trimmed = raw.trim();

    if trimmed.starts_with('{') {
        let value = serde_json::from_str::<serde_json::Value>(trimmed).ok()?;
        value.get("pid")?.as_u64().map(|pid| pid as u32)
    } else {
        trimmed.parse::<u32>().ok()
    }
}

fn local_providers() -> [&'static str; 5] {
    ["custom", "lmstudio", "ollama", "vllm", "llamacpp"]
}

fn resolve_api_key_env(base_url: &str) -> Option<&'static str> {
    [
        (r"openrouter\.ai", "OPENROUTER_API_KEY"),
        (r"anthropic\.com", "ANTHROPIC_API_KEY"),
        (r"openai\.com", "OPENAI_API_KEY"),
        (r"huggingface\.co", "HF_TOKEN"),
    ]
    .iter()
    .find_map(|(pattern, env_key)| {
        Regex::new(pattern)
            .ok()
            .filter(|re| re.is_match(base_url))
            .map(|_| *env_key)
    })
}

fn configure_model_env(
    envs: &mut Vec<(String, String)>,
    profile: Option<&str>,
    model_config: &ModelConfig,
) {
    if !local_providers().contains(&model_config.provider.as_str()) || model_config.base_url.is_empty() {
        return;
    }

    envs.push(("HERMES_INFERENCE_PROVIDER".into(), "custom".into()));
    envs.push((
        "OPENAI_BASE_URL".into(),
        model_config.base_url.trim_end_matches('/').to_string(),
    ));

    let env_map = read_env_map(profile);
    let resolved_key = resolve_api_key_env(&model_config.base_url)
        .and_then(|key| {
            env_map
                .iter()
                .find(|(env_key, _)| env_key == key)
                .map(|(_, value)| value.clone())
        })
        .or_else(|| {
            env_map
                .iter()
                .find(|(env_key, _)| env_key == "OPENAI_API_KEY")
                .map(|(_, value)| value.clone())
        })
        .unwrap_or_else(|| {
            if Regex::new(r"localhost|127\.0\.0\.1")
                .ok()
                .is_some_and(|re| re.is_match(&model_config.base_url))
            {
                NO_KEY_REQUIRED.to_string()
            } else {
                NO_KEY_REQUIRED.to_string()
            }
        });

    envs.push(("OPENAI_API_KEY".into(), resolved_key));
}

fn gateway_is_running(gateway_state: &GatewayState) -> bool {
    let state_pid = gateway_state.pid.lock().ok().and_then(|guard| *guard);
    if let Some(pid) = state_pid {
        if is_process_running(pid) {
            return true;
        }
        if let Ok(mut guard) = gateway_state.pid.lock() {
            *guard = None;
        }
    }

    read_gateway_pid_file().is_some_and(is_process_running)
}

#[tauri::command]
fn check_install() -> InstallStatus {
    let installed = hermes_python().exists() && hermes_script().exists();
    let configured = env_path(None).exists();
    let has_api_key = read_env_map(None).iter().any(|(key, value)| {
        matches!(
            key.as_str(),
            "OPENROUTER_API_KEY" | "ANTHROPIC_API_KEY" | "OPENAI_API_KEY"
        ) && !value.trim().is_empty()
    });
    let verified = if installed {
        hermes_command(&["--version"]).is_ok()
    } else {
        false
    };

    InstallStatus {
        installed,
        configured,
        has_api_key,
        verified,
    }
}

#[tauri::command]
fn get_hermes_version() -> Option<String> {
    hermes_command(&["--version"]).ok()
}

#[tauri::command]
fn get_model_config(profile: Option<String>) -> ModelConfig {
    load_model_config(profile.as_deref())
}

#[tauri::command]
fn set_model_config(
    provider: String,
    model: String,
    base_url: String,
    profile: Option<String>,
) -> Result<bool, String> {
    let profile = profile.as_deref();
    let mut changed = false;
    changed |= replace_config_value("provider", &provider, profile)?;
    changed |= replace_config_value("default", &model, profile)?;
    changed |= replace_config_value("base_url", &base_url, profile)?;
    Ok(changed)
}

#[tauri::command]
fn get_config(key: String, profile: Option<String>) -> Option<String> {
    read_config_value(&key, profile.as_deref())
}

#[tauri::command]
fn set_config(key: String, value: String, profile: Option<String>) -> Result<bool, String> {
    replace_config_value(&key, &value, profile.as_deref())
}

#[tauri::command]
fn get_env(profile: Option<String>) -> serde_json::Map<String, serde_json::Value> {
    let mut result = serde_json::Map::new();
    for (key, value) in read_env_map(profile.as_deref()) {
        result.insert(key, serde_json::Value::String(value));
    }
    result
}

#[tauri::command]
fn set_env(key: String, value: String, profile: Option<String>) -> Result<bool, String> {
    write_env_value(&key, &value, profile.as_deref())
}

#[tauri::command]
fn get_hermes_home(profile: Option<String>) -> String {
    profile_home(profile.as_deref()).to_string_lossy().to_string()
}

#[tauri::command]
fn list_models() -> Vec<ModelRecord> {
    let path = models_path();
    if let Ok(content) = fs::read_to_string(path) {
        serde_json::from_str::<Vec<ModelRecord>>(&content).unwrap_or_else(|_| default_models())
    } else {
        default_models()
    }
}

#[tauri::command]
fn list_profiles() -> Vec<ProfileInfo> {
    let active_name = get_active_profile_name();
    let mut profiles = Vec::new();

    let default_path = hermes_home();
    let (default_model, default_provider) = read_profile_config(&default_path);
    profiles.push(ProfileInfo {
        name: "default".to_string(),
        path: default_path.to_string_lossy().to_string(),
        is_default: true,
        is_active: active_name == "default",
        model: default_model,
        provider: default_provider,
        has_env: default_path.join(".env").exists(),
        has_soul: default_path.join("SOUL.md").exists(),
        skill_count: count_skills(&default_path),
        gateway_running: is_gateway_running(&default_path),
    });

    let profiles_dir = hermes_home().join("profiles");
    if let Ok(entries) = fs::read_dir(profiles_dir) {
        for entry in entries.flatten() {
            let profile_path = entry.path();
            if !profile_path.is_dir() {
                continue;
            }

            let has_config = profile_path.join("config.yaml").exists();
            let has_env = profile_path.join(".env").exists();
            if !has_config && !has_env {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            let (model, provider) = read_profile_config(&profile_path);
            profiles.push(ProfileInfo {
                name: name.clone(),
                path: profile_path.to_string_lossy().to_string(),
                is_default: false,
                is_active: active_name == name,
                model,
                provider,
                has_env,
                has_soul: profile_path.join("SOUL.md").exists(),
                skill_count: count_skills(&profile_path),
                gateway_running: is_gateway_running(&profile_path),
            });
        }
    }

    profiles
}

#[tauri::command]
fn create_profile(name: String, clone: bool) -> ActionResult {
    let mut args = vec!["profile", "create", name.as_str()];
    if clone {
        args.push("--clone");
    }

    match hermes_command(&args) {
        Ok(_) => ActionResult {
            success: true,
            error: None,
        },
        Err(error) => ActionResult {
            success: false,
            error: Some(error),
        },
    }
}

#[tauri::command]
fn delete_profile(name: String) -> ActionResult {
    if name == "default" {
        return ActionResult {
            success: false,
            error: Some("Cannot delete the default profile".to_string()),
        };
    }

    match hermes_command(&["profile", "delete", name.as_str(), "--yes"]) {
        Ok(_) => ActionResult {
            success: true,
            error: None,
        },
        Err(error) => ActionResult {
            success: false,
            error: Some(error),
        },
    }
}

#[tauri::command]
fn set_active_profile(name: String) -> bool {
    hermes_command(&["profile", "use", name.as_str()]).is_ok()
}

#[tauri::command]
fn read_soul(profile: Option<String>) -> String {
    read_file(&soul_path(profile.as_deref())).unwrap_or_default()
}

#[tauri::command]
fn write_soul(content: String, profile: Option<String>) -> bool {
    safe_write_file(&soul_path(profile.as_deref()), &content).is_ok()
}

#[tauri::command]
fn reset_soul(profile: Option<String>) -> String {
    let _ = safe_write_file(&soul_path(profile.as_deref()), DEFAULT_SOUL);
    DEFAULT_SOUL.to_string()
}

#[tauri::command]
fn read_memory(profile: Option<String>) -> MemoryState {
    let profile = profile.as_deref();
    let mem_file = read_file_state(&memory_path(profile));
    let user_file = read_file_state(&user_path(profile));
    let entries = parse_memory_entries(&mem_file.content);

    MemoryState {
        memory: MemoryDetail {
            content: mem_file.content.clone(),
            exists: mem_file.exists,
            last_modified: mem_file.last_modified,
            entries,
            char_count: mem_file.content.len(),
            char_limit: MEMORY_CHAR_LIMIT,
        },
        user: UserDetail {
            content: user_file.content.clone(),
            exists: user_file.exists,
            last_modified: user_file.last_modified,
            char_count: user_file.content.len(),
            char_limit: USER_CHAR_LIMIT,
        },
        stats: get_session_stats(profile),
    }
}

#[tauri::command]
fn add_memory_entry(content: String, profile: Option<String>) -> ActionResult {
    let profile = profile.as_deref();
    let current = read_file_state(&memory_path(profile));
    let mut entries = parse_memory_entries(&current.content);
    entries.push(MemoryEntry {
        index: entries.len(),
        content: content.trim().to_string(),
    });
    let new_content = serialize_memory_entries(&entries);

    if new_content.len() > MEMORY_CHAR_LIMIT {
        return ActionResult {
            success: false,
            error: Some(format!(
                "Would exceed memory limit ({}/{}) chars",
                new_content.len(),
                MEMORY_CHAR_LIMIT
            )),
        };
    }

    match safe_write_file(&memory_path(profile), &new_content) {
        Ok(_) => ActionResult {
            success: true,
            error: None,
        },
        Err(error) => ActionResult {
            success: false,
            error: Some(error),
        },
    }
}

#[tauri::command]
fn update_memory_entry(index: usize, content: String, profile: Option<String>) -> ActionResult {
    let profile = profile.as_deref();
    let current = read_file_state(&memory_path(profile));
    let mut entries = parse_memory_entries(&current.content);

    if index >= entries.len() {
        return ActionResult {
            success: false,
            error: Some("Entry not found".to_string()),
        };
    }

    entries[index].content = content.trim().to_string();
    let new_content = serialize_memory_entries(&entries);

    if new_content.len() > MEMORY_CHAR_LIMIT {
        return ActionResult {
            success: false,
            error: Some(format!(
                "Would exceed memory limit ({}/{}) chars",
                new_content.len(),
                MEMORY_CHAR_LIMIT
            )),
        };
    }

    match safe_write_file(&memory_path(profile), &new_content) {
        Ok(_) => ActionResult {
            success: true,
            error: None,
        },
        Err(error) => ActionResult {
            success: false,
            error: Some(error),
        },
    }
}

#[tauri::command]
fn remove_memory_entry(index: usize, profile: Option<String>) -> bool {
    let profile = profile.as_deref();
    let current = read_file_state(&memory_path(profile));
    let mut entries = parse_memory_entries(&current.content);
    if index >= entries.len() {
        return false;
    }
    entries.remove(index);
    safe_write_file(&memory_path(profile), &serialize_memory_entries(&entries)).is_ok()
}

#[tauri::command]
fn write_user_profile(content: String, profile: Option<String>) -> ActionResult {
    if content.len() > USER_CHAR_LIMIT {
        return ActionResult {
            success: false,
            error: Some(format!(
                "Exceeds limit ({}/{}) chars",
                content.len(),
                USER_CHAR_LIMIT
            )),
        };
    }

    match safe_write_file(&user_path(profile.as_deref()), &content) {
        Ok(_) => ActionResult {
            success: true,
            error: None,
        },
        Err(error) => ActionResult {
            success: false,
            error: Some(error),
        },
    }
}

#[tauri::command]
fn list_sessions(limit: Option<i64>, offset: Option<i64>) -> Vec<SessionSummary> {
    let Some(conn) = open_db_readonly(None) else {
        return vec![];
    };

    let limit = limit.unwrap_or(30);
    let offset = offset.unwrap_or(0);
    let mut stmt = match conn.prepare(
        "SELECT id, source, started_at, ended_at, message_count, model, title
         FROM sessions
         ORDER BY started_at DESC
         LIMIT ?1 OFFSET ?2",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return vec![],
    };

    let rows = stmt.query_map(params![limit, offset], |row| {
        Ok(SessionSummary {
            id: row.get(0)?,
            source: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            started_at: row.get(2)?,
            ended_at: row.get(3)?,
            message_count: row.get(4)?,
            model: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
            title: row.get(6)?,
            preview: String::new(),
        })
    });

    rows.map(|mapped| mapped.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

#[tauri::command]
fn list_cached_sessions(limit: Option<i64>, offset: Option<i64>) -> Vec<CachedSession> {
    list_sessions(limit, offset)
        .into_iter()
        .map(|session| CachedSession {
            id: session.id,
            title: session.title.unwrap_or_else(|| "New conversation".to_string()),
            started_at: session.started_at,
            source: session.source,
            message_count: session.message_count,
            model: session.model,
        })
        .collect()
}

#[tauri::command]
fn sync_session_cache() -> Vec<CachedSession> {
    list_cached_sessions(Some(50), Some(0))
}

#[tauri::command]
fn get_session_messages(session_id: String) -> Vec<SessionMessage> {
    let Some(conn) = open_db_readonly(None) else {
        return vec![];
    };

    let mut stmt = match conn.prepare(
        "SELECT id, role, content, timestamp
         FROM messages
         WHERE session_id = ?1
           AND role IN ('user', 'assistant')
           AND content IS NOT NULL
         ORDER BY timestamp, id",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return vec![],
    };

    let rows = stmt.query_map(params![session_id], |row| {
        Ok(SessionMessage {
            id: row.get(0)?,
            role: row.get::<_, String>(1)?,
            content: row.get::<_, String>(2)?,
            timestamp: row.get(3)?,
        })
    });

    rows.map(|mapped| mapped.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

#[tauri::command]
fn search_sessions(query: String, limit: Option<i64>) -> Vec<SearchResult> {
    let Some(conn) = open_db_readonly(None) else {
        return vec![];
    };

    let table_exists: bool = conn
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sqlite_master WHERE type='table' AND name='messages_fts'
            )",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);
    if !table_exists {
        return vec![];
    }

    let sanitized = query
        .trim()
        .split_whitespace()
        .filter(|word| !word.is_empty())
        .map(|word| format!("\"{}\"*", word.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" ");

    if sanitized.is_empty() {
        return vec![];
    }

    let mut stmt = match conn.prepare(
        "SELECT DISTINCT
            m.session_id,
            s.title,
            s.started_at,
            s.source,
            s.message_count,
            s.model,
            snippet(messages_fts, 0, '<<', '>>', '...', 40) AS snippet
         FROM messages_fts
         JOIN messages m ON m.id = messages_fts.rowid
         JOIN sessions s ON s.id = m.session_id
         WHERE messages_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return vec![],
    };

    let rows = stmt.query_map(params![sanitized, limit.unwrap_or(20)], |row| {
        Ok(SearchResult {
            session_id: row.get(0)?,
            title: row.get(1)?,
            started_at: row.get(2)?,
            source: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            message_count: row.get(4)?,
            model: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
            snippet: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
        })
    });

    rows.map(|mapped| mapped.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

#[tauri::command]
fn update_session_title(session_id: String, title: String) {
    let Some(conn) = open_db_readwrite(None) else {
        return;
    };
    let _ = conn.execute(
        "UPDATE sessions SET title = ?1 WHERE id = ?2",
        params![title, session_id],
    );
}

#[tauri::command]
fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
fn claw3d_status() -> Claw3dStatus {
    Claw3dStatus {
        cloned: false,
        installed: false,
        dev_server_running: false,
        adapter_running: false,
        port: 3000,
        port_in_use: false,
        ws_url: "ws://localhost:18789".into(),
        running: false,
        error: String::new(),
    }
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    open::that(url).map_err(|err| err.to_string())
}

#[tauri::command]
fn open_office_window(app: tauri::AppHandle, url: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("office") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(
        &app,
        "office",
        WebviewUrl::External(url.parse().map_err(|err: url::ParseError| err.to_string())?),
    )
    .title("Hermes Office")
    .inner_size(1280.0, 800.0)
    .build()
    .map(|_| ())
    .map_err(|err| err.to_string())
}

#[tauri::command]
fn start_gateway(
    gateway_state: State<'_, GatewayState>,
    profile: Option<String>,
) -> bool {
    if gateway_is_running(&gateway_state) {
        return false;
    }

    let profile = profile.as_deref();
    let mut command = Command::new(hermes_python());
    command
        .arg(hermes_script())
        .arg("gateway")
        .current_dir(hermes_repo())
        .env("PATH", get_enhanced_path())
        .env("HERMES_HOME", hermes_home())
        .env("HOME", home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .env("API_SERVER_ENABLED", "true")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    for (key, value) in read_env_map(profile) {
        if !value.is_empty() {
            command.env(key, value);
        }
    }

    match command.spawn() {
        Ok(child) => {
            if let Ok(mut pid_lock) = gateway_state.pid.lock() {
                *pid_lock = Some(child.id());
            }
            true
        }
        Err(_) => false,
    }
}

#[tauri::command]
fn stop_gateway(gateway_state: State<'_, GatewayState>) -> bool {
    let pid = gateway_state
        .pid
        .lock()
        .ok()
        .and_then(|mut guard| guard.take())
        .or_else(read_gateway_pid_file);

    if let Some(pid) = pid {
        kill_process(pid, "-TERM");
        return true;
    }
    false
}

#[tauri::command]
fn gateway_status(gateway_state: State<'_, GatewayState>) -> bool {
    gateway_is_running(&gateway_state)
}

#[tauri::command]
fn send_message(
    app: tauri::AppHandle,
    chat_state: State<'_, ChatState>,
    message: String,
    profile: Option<String>,
    resume_session_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let profile_ref = profile.as_deref();
    let model_config = load_model_config(profile_ref);
    let profile_env = read_env_map(profile_ref);

    chat_state.aborted.store(false, Ordering::SeqCst);

    let mut args = vec![hermes_script().to_string_lossy().to_string()];
    if let Some(profile_name) = profile_ref.filter(|name| *name != "default") {
        args.push("-p".into());
        args.push(profile_name.to_string());
    }
    args.extend([
        "chat".into(),
        "-q".into(),
        message.clone(),
        "-Q".into(),
        "--source".into(),
        "desktop".into(),
    ]);

    if let Some(session_id) = &resume_session_id {
        args.push("--resume".into());
        args.push(session_id.clone());
    }
    if !model_config.model.is_empty() {
        args.push("-m".into());
        args.push(model_config.model.clone());
    }

    let mut command = Command::new(hermes_python());
    command
        .args(args)
        .current_dir(hermes_repo())
        .env("PATH", get_enhanced_path())
        .env("HERMES_HOME", hermes_home())
        .env("HOME", home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .env("PYTHONUNBUFFERED", "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    for (key, value) in profile_env {
        if !value.is_empty() {
            command.env(key, value);
        }
    }
    let mut extra_envs = Vec::new();
    configure_model_env(&mut extra_envs, profile_ref, &model_config);
    for (key, value) in extra_envs {
        command.env(key, value);
    }

    let mut child = command.spawn().map_err(|err| err.to_string())?;

    if let Ok(mut pid_lock) = chat_state.pid.lock() {
        *pid_lock = Some(child.id());
    }

    let stdout = child.stdout.take().ok_or_else(|| "Failed to capture stdout".to_string())?;
    let stderr = child.stderr.take().ok_or_else(|| "Failed to capture stderr".to_string())?;

    let app_stdout = app.clone();
    let out_handle = std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(stdout);
        let mut full_response = String::new();
        let mut session_id = String::new();

        for line_result in reader.lines() {
            let Ok(line) = line_result else { continue };
            if let Some(rest) = line.trim().strip_prefix("session_id:") {
                session_id = rest.trim().to_string();
                continue;
            }
            if line.trim().is_empty() {
                continue;
            }
            let chunk = format!("{line}\n");
            full_response.push_str(&chunk);
            let _ = app_stdout.emit("chat-chunk", chunk);
        }

        (full_response, session_id)
    });

    let app_stderr = app.clone();
    let err_handle = std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(stderr);
        let mut captured = String::new();

        for line_result in reader.lines() {
            let Ok(line) = line_result else { continue };
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.contains("UserWarning")
                || trimmed.contains("FutureWarning")
            {
                continue;
            }

            if Regex::new(r"❌|⚠️|Error|Traceback|error|failed|denied|unauthorized|invalid")
                .ok()
                .is_some_and(|re| re.is_match(trimmed))
            {
                let chunk = format!("{line}\n");
                let _ = app_stderr.emit("chat-chunk", chunk.clone());
                captured.push_str(&chunk);
            } else {
                captured.push_str(&line);
                captured.push('\n');
            }
        }

        captured
    });

    let status = child.wait().map_err(|err| err.to_string())?;
    let (response, session_id) = out_handle
        .join()
        .map_err(|_| "Failed to join stdout thread".to_string())?;
    let stderr_output = err_handle
        .join()
        .map_err(|_| "Failed to join stderr thread".to_string())?;

    if let Ok(mut pid_lock) = chat_state.pid.lock() {
        *pid_lock = None;
    }

    let aborted = chat_state.aborted.swap(false, Ordering::SeqCst);
    if aborted {
        let _ = app.emit("chat-done", session_id.clone());
        return Ok(serde_json::json!({
            "response": response,
            "sessionId": if session_id.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(session_id) }
        }));
    }

    if status.success() || !response.trim().is_empty() {
        let _ = app.emit("chat-done", session_id.clone());
        Ok(serde_json::json!({
            "response": response,
            "sessionId": if session_id.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(session_id) }
        }))
    } else {
        let message = if stderr_output.trim().is_empty() {
            format!("Hermes exited with status {}", status)
        } else {
            stderr_output.trim().to_string()
        };
        let _ = app.emit("chat-error", message.clone());
        Err(message)
    }
}

#[tauri::command]
fn abort_chat(chat_state: State<'_, ChatState>) {
    chat_state.aborted.store(true, Ordering::SeqCst);
    if let Ok(mut pid_lock) = chat_state.pid.lock() {
        if let Some(pid) = *pid_lock {
            kill_process(pid, "-TERM");
            *pid_lock = None;
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ChatState::default())
        .manage(GatewayState::default())
        .invoke_handler(tauri::generate_handler![
            abort_chat,
            add_memory_entry,
            check_install,
            claw3d_status,
            create_profile,
            delete_profile,
            get_app_version,
            get_config,
            get_env,
            get_hermes_home,
            get_hermes_version,
            get_model_config,
            get_session_messages,
            gateway_status,
            list_cached_sessions,
            list_models,
            list_profiles,
            list_sessions,
            open_external,
            open_office_window,
            read_memory,
            read_soul,
            remove_memory_entry,
            reset_soul,
            search_sessions,
            send_message,
            set_active_profile,
            set_config,
            set_env,
            set_model_config,
            start_gateway,
            stop_gateway,
            sync_session_cache,
            update_memory_entry,
            update_session_title,
            write_soul,
            write_user_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
