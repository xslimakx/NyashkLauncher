//! GitHub Releases client (with ETag cache), downloader and installer.

use crate::model::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Asset {
    pub name: String,
    #[serde(default)]
    pub size: u64,
    pub browser_download_url: String,
    #[serde(default)]
    pub digest: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Release {
    pub tag_name: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

impl Release {
    /// The zip built for this OS, if the release has one.
    pub fn asset(&self) -> Option<&Asset> {
        self.assets.iter().find(|a| OS.matches_asset(&a.name))
    }
    /// Same, for a repo that holds several plugins: prefer the zip named after the bundle.
    pub fn asset_for(&self, bundle: &str) -> Option<&Asset> {
        let stem = bundle.trim_end_matches(".vst3").to_ascii_lowercase();
        let cands: Vec<&Asset> = self.assets.iter().filter(|a| OS.matches_asset(&a.name)).collect();
        cands
            .iter()
            .find(|a| a.name.to_ascii_lowercase().starts_with(&stem))
            .copied()
            .or(if cands.len() == 1 { cands.first().copied() } else { None })
    }
    pub fn date(&self) -> &str {
        self.published_at.as_deref().map(|s| s.get(..10).unwrap_or(s)).unwrap_or("")
    }
}

#[derive(Serialize, Deserialize, Default)]
struct CacheFile {
    etag: String,
    releases: Vec<Release>,
}

pub fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .try_proxy_from_env(true)
        .user_agent(concat!("NyashkLauncher/", env!("CARGO_PKG_VERSION")))
        .build()
}

/// Fetch all releases of a repo. Uses `If-None-Match`, so unchanged repos cost
/// no API rate limit and no bandwidth (HTTP 304 -> cached copy).
pub fn fetch_releases(repo: &str, token: &str) -> Result<Vec<Release>, String> {
    let cache_path = cache_dir().join(format!("releases_{}.json", repo.replace('/', "_")));
    let cached: Option<CacheFile> =
        std::fs::read(&cache_path).ok().and_then(|b| serde_json::from_slice(&b).ok());

    let mut req = agent()
        .get(&format!("https://api.github.com/repos/{repo}/releases?per_page=50"))
        .set("Accept", "application/vnd.github+json");
    if !token.trim().is_empty() {
        req = req.set("Authorization", &format!("Bearer {}", token.trim()));
    }
    if let Some(c) = &cached {
        if !c.etag.is_empty() {
            req = req.set("If-None-Match", &c.etag);
        }
    }

    let offline = |e: String| match &cached {
        Some(c) => Ok(c.releases.clone()), // offline: fall back to the last known list
        None => Err(e),
    };

    // one automatic retry: transient proxy / network hiccups are common on flaky connections
    let mut result = req.clone().call();
    if matches!(result, Err(ureq::Error::Transport(_))) {
        std::thread::sleep(Duration::from_millis(600));
        result = req.clone().call();
    }
    match result {
        Ok(resp) => {
            let etag = resp.header("ETag").unwrap_or("").to_string();
            let parsed = resp
                .into_string()
                .map_err(|e| e.to_string())
                .and_then(|b| serde_json::from_str::<Vec<Release>>(&b).map_err(|e| e.to_string()));
            match parsed {
                Ok(releases) => {
                    let cf = CacheFile { etag, releases: releases.clone() };
                    if let Ok(b) = serde_json::to_vec(&cf) {
                        let _ = std::fs::write(&cache_path, b);
                    }
                    Ok(releases)
                }
                Err(_) => offline(format!("{repo}: GitHub sent an unreadable answer, try again")),
            }
        }
        Err(ureq::Error::Status(304, _)) => Ok(cached.map(|c| c.releases).unwrap_or_default()),
        Err(ureq::Error::Status(code, _)) if code == 403 || code == 429 => offline(format!(
            "{repo}: GitHub rate limit reached (HTTP {code}). Add a token in Settings or try later."
        )),
        Err(ureq::Error::Status(404, _)) => Ok(Vec::new()),
        Err(ureq::Error::Status(code, _)) => offline(format!("{repo}: GitHub answered HTTP {code}")),
        Err(_) => offline(format!("{repo}: cannot reach GitHub, check your internet connection")),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Download `asset` into the cache folder. `progress(done, total)` is called as data arrives.
pub fn download(
    asset: &Asset,
    token: &str,
    mut progress: impl FnMut(u64, u64),
) -> Result<std::path::PathBuf, String> {
    let _ = token; // public release assets need no authentication
    let resp = agent()
        .get(&asset.browser_download_url)
        .timeout(Duration::from_secs(600))
        .call()
        .map_err(|e| format!("download failed: {e}"))?;
    let total = resp
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(asset.size);

    let dl_dir = cache_dir().join("downloads");
    std::fs::create_dir_all(&dl_dir).map_err(|e| e.to_string())?;
    let part = dl_dir.join(format!("{}.part", asset.name));
    let mut file = std::fs::File::create(&part).map_err(|e| format!("cannot write cache: {e}"))?;
    let mut reader = resp.into_reader();
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut done = 0u64;
    loop {
        let n = reader.read(&mut buf).map_err(|e| format!("download interrupted: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        done += n as u64;
        progress(done, total);
    }
    drop(file);

    if let Some(expected) = asset.digest.as_deref().and_then(|d| d.strip_prefix("sha256:")) {
        let got = hex(hasher.finalize().as_slice());
        if !got.eq_ignore_ascii_case(expected) {
            let _ = std::fs::remove_file(&part);
            return Err("checksum mismatch: the download is corrupted, please try again".into());
        }
    }
    let path = dl_dir.join(&asset.name);
    let _ = std::fs::remove_file(&path);
    std::fs::rename(&part, &path).map_err(|e| e.to_string())?;
    Ok(path)
}

fn io_hint(e: &std::io::Error, dir: &Path) -> String {
    match e.kind() {
        std::io::ErrorKind::PermissionDenied => format!(
            "no permission to write to {} (or the plugin is loaded by your DAW). \
             Close the DAW, or choose another folder in Settings.",
            dir.display()
        ),
        _ => e.to_string(),
    }
}

/// Unpack the zip into `<dir>/<bundle>`, swapping atomically: the old copy is only
/// removed once the new one is complete, and restored if anything fails.
pub fn install_zip(zip_path: &Path, def: &PluginDef, dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| io_hint(&e, dir))?;
    let staging = dir.join(format!(".{}.new", def.bundle));
    let backup = dir.join(format!(".{}.old", def.bundle));
    let target = dir.join(def.bundle);
    let _ = std::fs::remove_dir_all(&staging);
    let _ = std::fs::remove_dir_all(&backup);

    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("bad archive: {e}"))?;
    let mut files = 0;
    let extract = (|| -> Result<(), String> {
        std::fs::create_dir_all(&staging).map_err(|e| io_hint(&e, dir))?;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
            let Some(rel) = entry.enclosed_name() else { continue }; // blocks ../ traversal
            let Ok(inner) = rel.strip_prefix(def.bundle) else { continue };
            let out = staging.join(inner);
            if entry.is_dir() {
                std::fs::create_dir_all(&out).map_err(|e| io_hint(&e, dir))?;
                continue;
            }
            if let Some(p) = out.parent() {
                std::fs::create_dir_all(p).map_err(|e| io_hint(&e, dir))?;
            }
            let mut f = std::fs::File::create(&out).map_err(|e| io_hint(&e, dir))?;
            std::io::copy(&mut entry, &mut f).map_err(|e| io_hint(&e, dir))?;
            #[cfg(unix)]
            if let Some(mode) = entry.unix_mode() {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&out, std::fs::Permissions::from_mode(mode));
            }
            files += 1;
        }
        if files == 0 {
            return Err(format!("the archive does not contain {}", def.bundle));
        }
        Ok(())
    })();
    if let Err(e) = extract {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(e);
    }

    if target.exists() {
        if let Err(e) = std::fs::rename(&target, &backup) {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(io_hint(&e, dir));
        }
    }
    if let Err(e) = std::fs::rename(&staging, &target) {
        let _ = std::fs::rename(&backup, &target); // put the old version back
        let _ = std::fs::remove_dir_all(&staging);
        return Err(io_hint(&e, dir));
    }
    let _ = std::fs::remove_dir_all(&backup);
    Ok(())
}

pub fn uninstall(def: &PluginDef, dir: &Path) -> Result<(), String> {
    let target = dir.join(def.bundle);
    if target.exists() {
        std::fs::remove_dir_all(&target).map_err(|e| io_hint(&e, dir))?;
    }
    Ok(())
}

/// Replace the running executable with the one inside the downloaded zip.
pub fn self_update(zip_path: &Path) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let want = if cfg!(windows) { "NyashkLauncher.exe" } else { "NyashkLauncher" };
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("bad archive: {e}"))?;
    let mut entry = zip.by_name(want).map_err(|_| "the archive has no launcher binary".to_string())?;
    let new = exe.with_extension("new");
    let hint = |e: std::io::Error| format!("cannot replace the launcher ({e}). Is its folder writable?");
    {
        let mut f = std::fs::File::create(&new).map_err(hint)?;
        std::io::copy(&mut entry, &mut f).map_err(hint)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&new, std::fs::Permissions::from_mode(0o755));
        std::fs::rename(&new, &exe).map_err(hint)?;
    }
    #[cfg(windows)]
    {
        // a running .exe cannot be overwritten, but it can be renamed
        let old = exe.with_extension("old");
        let _ = std::fs::remove_file(&old);
        std::fs::rename(&exe, &old).map_err(hint)?;
        if let Err(e) = std::fs::rename(&new, &exe) {
            let _ = std::fs::rename(&old, &exe);
            return Err(hint(e));
        }
    }
    Ok(())
}

/// Download `plugins.json` (and the preview pictures it names) into the cache.
/// Returns true if the list changed, so the launcher can ask for a restart.
pub fn refresh_registry() -> Result<bool, String> {
    let url = format!("{REGISTRY_BASE}plugins.json");
    let body = agent()
        .get(&url)
        .call()
        .map_err(|e| format!("plugin list: {e}"))?
        .into_string()
        .map_err(|e| e.to_string())?;
    let parsed: serde_json::Value = serde_json::from_str(&body).map_err(|e| format!("plugin list is invalid: {e}"))?;
    let _ = std::fs::create_dir_all(cache_dir().join("shots"));
    // pictures first, so the list never refers to a picture that is not on disk
    if let Some(arr) = parsed.get("plugins").and_then(|p| p.as_array()) {
        for p in arr {
            for s in p.get("shots").and_then(|s| s.as_array()).into_iter().flatten() {
                let Some(u) = s.get("url").and_then(|u| u.as_str()) else { continue };
                let u = if u.starts_with("http") { u.to_string() } else { format!("{REGISTRY_BASE}{}", u.trim_start_matches('/')) };
                let path = shot_cache_path(&u);
                if path.exists() {
                    continue;
                }
                let mut bytes = Vec::new();
                if let Ok(r) = agent().get(&u).call() {
                    if r.into_reader().take(8 * 1024 * 1024).read_to_end(&mut bytes).is_ok() && !bytes.is_empty() {
                        let _ = std::fs::write(&path, bytes);
                    }
                }
            }
        }
    }
    let path = cache_dir().join("plugins.json");
    let old = std::fs::read_to_string(&path).unwrap_or_default();
    if old == body {
        return Ok(false);
    }
    std::fs::write(&path, &body).map_err(|e| e.to_string())?;
    // the list in memory came from the old file (or the built-in one): changed only if it differs in content
    Ok(true)
}
