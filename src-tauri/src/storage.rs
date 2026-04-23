use crate::settings::Settings;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub fn app_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("Sneaker");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn settings_path() -> PathBuf { app_dir().join("settings.json") }

pub fn default_notes_dir() -> PathBuf {
    let d = app_dir().join("notes");
    let _ = fs::create_dir_all(&d);
    d
}

pub fn notes_dir(s: &Settings) -> PathBuf {
    if s.notes_dir.is_empty() {
        default_notes_dir()
    } else {
        let p = PathBuf::from(&s.notes_dir);
        let _ = fs::create_dir_all(&p);
        p
    }
}

pub fn load_settings() -> Settings {
    let p = settings_path();
    if let Ok(txt) = fs::read_to_string(&p) {
        if let Ok(s) = serde_json::from_str::<Settings>(&txt) {
            return s;
        }
    }
    Settings::default()
}

pub fn save_settings(s: &Settings) -> Result<()> {
    let p = settings_path();
    let txt = serde_json::to_string_pretty(s)?;
    fs::write(p, txt)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteMeta {
    pub id: String,
    pub name: String,
    pub path: String,
    pub encrypted: bool,
    pub modified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct NoteSide {
    encrypted: bool,
    #[serde(default)]
    salt: String,
    #[serde(default)]
    nonce: String,
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if "\\/:*?\"<>|".contains(c) { '_' } else { c })
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn list_notes(s: &Settings) -> Result<Vec<NoteMeta>> {
    let dir = notes_dir(s);
    let mut out = Vec::new();
    if !dir.exists() { return Ok(out); }
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let p = entry.path();
        if !p.is_file() { continue; }
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "md" | "txt" | "enc") { continue; }
        let name = p.file_stem().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let side = read_side(&p);
        let modified = entry.metadata()?.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        out.push(NoteMeta {
            id: name.clone(),
            name,
            path: p.to_string_lossy().to_string(),
            encrypted: side.encrypted,
            modified,
        });
    }
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(out)
}

fn side_path(p: &Path) -> PathBuf { p.with_extension("meta.json") }

fn read_side(p: &Path) -> NoteSide {
    let sp = side_path(p);
    fs::read_to_string(&sp).ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn write_side(p: &Path, side: &NoteSide) -> Result<()> {
    let sp = side_path(p);
    fs::write(sp, serde_json::to_string(side)?)?;
    Ok(())
}

fn find_note(s: &Settings, id: &str) -> Option<PathBuf> {
    let dir = notes_dir(s);
    for ext in ["md", "txt", "enc"] {
        let p = dir.join(format!("{}.{}", id, ext));
        if p.exists() { return Some(p); }
    }
    None
}

pub fn create_note(s: &Settings, name: &str) -> Result<NoteMeta> {
    let dir = notes_dir(s);
    let sname = sanitize(name);
    let mut p = dir.join(format!("{}.md", sname));
    let mut i = 1;
    while p.exists() {
        p = dir.join(format!("{}-{}.md", sname, i));
        i += 1;
    }
    fs::write(&p, "")?;
    let id = p.file_stem().unwrap().to_string_lossy().to_string();
    Ok(NoteMeta {
        id: id.clone(),
        name: id,
        path: p.to_string_lossy().into(),
        encrypted: false,
        modified: now(),
    })
}

pub fn delete_note(s: &Settings, id: &str) -> Result<()> {
    if let Some(p) = find_note(s, id) {
        let _ = fs::remove_file(&p);
        let _ = fs::remove_file(side_path(&p));
    }
    Ok(())
}

pub fn rename_note(s: &Settings, id: &str, new_name: &str) -> Result<NoteMeta> {
    let p = find_note(s, id).ok_or_else(|| anyhow!("not found"))?;
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("md").to_string();
    let dir = p.parent().unwrap().to_path_buf();
    let new_name = sanitize(new_name);
    let mut np = dir.join(format!("{}.{}", new_name, ext));
    let mut i = 1;
    while np.exists() && np != p {
        np = dir.join(format!("{}-{}.{}", new_name, i, ext));
        i += 1;
    }
    fs::rename(&p, &np)?;
    let old_side = side_path(&p);
    if old_side.exists() {
        let _ = fs::rename(&old_side, side_path(&np));
    }
    let id = np.file_stem().unwrap().to_string_lossy().to_string();
    let side = read_side(&np);
    Ok(NoteMeta {
        id: id.clone(),
        name: id,
        path: np.to_string_lossy().into(),
        encrypted: side.encrypted,
        modified: now(),
    })
}

pub fn read_note(s: &Settings, id: &str, password: Option<&str>) -> Result<String> {
    let p = find_note(s, id).ok_or_else(|| anyhow!("not found"))?;
    let side = read_side(&p);
    if side.encrypted {
        let pw = password.ok_or_else(|| anyhow!("password required"))?;
        let cipher = fs::read(&p)?;
        let salt = base64_decode(&side.salt)?;
        let nonce = base64_decode(&side.nonce)?;
        decrypt(&cipher, pw.as_bytes(), &salt, &nonce)
    } else {
        Ok(fs::read_to_string(&p)?)
    }
}

pub fn save_note(s: &Settings, id: &str, content: &str, password: Option<&str>) -> Result<()> {
    let p = find_note(s, id).ok_or_else(|| anyhow!("not found"))?;
    let side = read_side(&p);
    if side.encrypted {
        let pw = password.ok_or_else(|| anyhow!("password required"))?;
        let salt = base64_decode(&side.salt)?;
        let (cipher, nonce) = encrypt(content.as_bytes(), pw.as_bytes(), &salt)?;
        fs::write(&p, cipher)?;
        let mut new_side = side.clone();
        new_side.nonce = base64_encode(&nonce);
        write_side(&p, &new_side)?;
    } else {
        fs::write(&p, content)?;
    }
    Ok(())
}

pub fn import_file(s: &Settings, path: &str) -> Result<NoteMeta> {
    let src = PathBuf::from(path);
    if !src.exists() { return Err(anyhow!("file not found")); }
    let content = fs::read_to_string(&src).unwrap_or_default();
    let name = src.file_stem().and_then(|n| n.to_str()).unwrap_or("imported").to_string();
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("md").to_string();
    let dir = notes_dir(s);
    let sname = sanitize(&name);
    let mut p = dir.join(format!("{}.{}", sname, ext));
    let mut i = 1;
    while p.exists() {
        p = dir.join(format!("{}-{}.{}", sname, i, ext));
        i += 1;
    }
    fs::write(&p, content)?;
    let id = p.file_stem().unwrap().to_string_lossy().to_string();
    Ok(NoteMeta {
        id: id.clone(),
        name: id,
        path: p.to_string_lossy().into(),
        encrypted: false,
        modified: now(),
    })
}

pub fn set_note_encryption(s: &Settings, id: &str, encrypt_flag: bool, password: &str) -> Result<()> {
    let p = find_note(s, id).ok_or_else(|| anyhow!("not found"))?;
    let side = read_side(&p);
    if encrypt_flag && !side.encrypted {
        let plain = fs::read_to_string(&p).unwrap_or_default();
        let salt = random_bytes(16);
        let (cipher, nonce) = encrypt(plain.as_bytes(), password.as_bytes(), &salt)?;
        let new_path = p.with_extension("enc");
        fs::write(&new_path, cipher)?;
        let _ = fs::remove_file(&p);
        let new_side = NoteSide { encrypted: true, salt: base64_encode(&salt), nonce: base64_encode(&nonce) };
        write_side(&new_path, &new_side)?;
    } else if !encrypt_flag && side.encrypted {
        let cipher = fs::read(&p)?;
        let salt = base64_decode(&side.salt)?;
        let nonce = base64_decode(&side.nonce)?;
        let plain = decrypt(&cipher, password.as_bytes(), &salt, &nonce)?;
        let new_path = p.with_extension("md");
        fs::write(&new_path, plain)?;
        let _ = fs::remove_file(&p);
        let _ = fs::remove_file(side_path(&p));
    }
    Ok(())
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn random_bytes(n: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut v = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut v);
    v
}

fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32]> {
    use argon2::Argon2;
    let mut out = [0u8; 32];
    Argon2::default()
        .hash_password_into(password, salt, &mut out)
        .map_err(|e| anyhow!("derive: {}", e))?;
    Ok(out)
}

pub fn encrypt(plain: &[u8], password: &[u8], salt: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    use chacha20poly1305::aead::{Aead, KeyInit};
    use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
    let key = derive_key(password, salt)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let nonce_bytes = random_bytes(12);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher.encrypt(nonce, plain).map_err(|e| anyhow!("enc: {}", e))?;
    Ok((ct, nonce_bytes))
}

pub fn decrypt(cipher: &[u8], password: &[u8], salt: &[u8], nonce: &[u8]) -> Result<String> {
    use chacha20poly1305::aead::{Aead, KeyInit};
    use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
    let key = derive_key(password, salt)?;
    let cc = ChaCha20Poly1305::new(Key::from_slice(&key));
    let n = Nonce::from_slice(nonce);
    let pt = cc.decrypt(n, cipher).map_err(|_| anyhow!("密码错误"))?;
    Ok(String::from_utf8(pt).unwrap_or_default())
}

pub fn base64_encode(b: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(b)
}
pub fn base64_decode(s: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.decode(s)?)
}

pub fn hash_password(pw: &str) -> Result<(String, String)> {
    let salt = random_bytes(16);
    let key = derive_key(pw.as_bytes(), &salt)?;
    Ok((base64_encode(&key), base64_encode(&salt)))
}

pub fn verify_password(pw: &str, hash_b64: &str, salt_b64: &str) -> Result<bool> {
    let salt = base64_decode(salt_b64)?;
    let hash = base64_decode(hash_b64)?;
    let key = derive_key(pw.as_bytes(), &salt)?;
    Ok(key.as_slice() == hash.as_slice())
}
