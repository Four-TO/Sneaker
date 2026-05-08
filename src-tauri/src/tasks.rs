use crate::storage::app_dir;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub notes: String,
    pub status: String, // "todo" | "working" | "paused" | "done"
    pub pinned: bool,
    #[serde(default)]
    pub pinned_bottom: bool,
    pub created_at: u64,
    #[serde(default)]
    pub completed_at: Option<u64>,
    #[serde(default)]
    pub order: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskPatch {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
    pub pinned: Option<bool>,
    pub pinned_bottom: Option<bool>,
    pub order: Option<i64>,
}

fn file_path() -> PathBuf { app_dir().join("tasks.json") }

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn load_all() -> Vec<Task> {
    let p = file_path();
    if let Ok(txt) = fs::read_to_string(&p) {
        if let Ok(v) = serde_json::from_str::<Vec<Task>>(&txt) {
            return v;
        }
    }
    Vec::new()
}

pub fn save_all(tasks: &[Task]) -> Result<()> {
    fs::write(file_path(), serde_json::to_string_pretty(tasks)?)?;
    Ok(())
}

pub fn create(title: &str) -> Result<Task> {
    let mut tasks = load_all();
    let title = title.trim();
    if title.is_empty() { return Err(anyhow!("empty title")); }
    let mut pinned = false;
    let mut status = "todo".to_string();
    let mut clean = title.to_string();
    if let Some(rest) = clean.strip_prefix('!') {
        pinned = true;
        clean = rest.trim().to_string();
    } else if let Some(rest) = clean.strip_prefix('>') {
        status = "working".into();
        clean = rest.trim().to_string();
    }
    if clean.is_empty() { return Err(anyhow!("empty title after prefix")); }
    let min_order = tasks.iter().map(|t| t.order).min().unwrap_or(0);
    let t = Task {
        id: uuid::Uuid::new_v4().to_string(),
        title: clean,
        notes: String::new(),
        status,
        pinned,
        pinned_bottom: false,
        created_at: now(),
        completed_at: None,
        order: min_order - 1,
    };
    tasks.insert(0, t.clone());
    save_all(&tasks)?;
    Ok(t)
}

pub fn update(id: &str, patch: TaskPatch) -> Result<Task> {
    let mut tasks = load_all();
    let idx = tasks.iter().position(|t| t.id == id).ok_or_else(|| anyhow!("not found"))?;
    let t = &mut tasks[idx];
    if let Some(v) = patch.title { t.title = v; }
    if let Some(v) = patch.notes { t.notes = v; }
    if let Some(v) = patch.pinned {
        t.pinned = v;
        if v { t.pinned_bottom = false; }
    }
    if let Some(v) = patch.pinned_bottom {
        t.pinned_bottom = v;
        if v { t.pinned = false; }
    }
    if let Some(v) = patch.order { t.order = v; }
    if let Some(v) = patch.status {
        if v == "done" && t.status != "done" { t.completed_at = Some(now()); }
        if v != "done" { t.completed_at = None; }
        t.status = v;
    }
    let out = t.clone();
    save_all(&tasks)?;
    Ok(out)
}

pub fn delete(id: &str) -> Result<()> {
    let mut tasks = load_all();
    tasks.retain(|t| t.id != id);
    save_all(&tasks)?;
    Ok(())
}

pub fn toggle(id: &str) -> Result<Task> {
    let mut tasks = load_all();
    let idx = tasks.iter().position(|t| t.id == id).ok_or_else(|| anyhow!("not found"))?;
    let t = &mut tasks[idx];
    if t.status == "done" {
        t.status = "todo".into();
        t.completed_at = None;
    } else {
        t.status = "done".into();
        t.completed_at = Some(now());
    }
    let out = t.clone();
    save_all(&tasks)?;
    Ok(out)
}
