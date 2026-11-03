//! Fetch by URL into a content cache, verify or record the sha256, and hand back a
//! path. The lock file is the registry's memory of which bytes a run used.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Lock {
    #[serde(default)]
    pub files: BTreeMap<String, Entry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub url: String,
    pub sha256: String,
    pub bytes: u64,
    pub fetched: String,
}

pub struct Fetcher {
    pub cache_dir: PathBuf,
    pub lock_path: PathBuf,
    pub lock: Lock,
}

impl Fetcher {
    pub fn new(cache_dir: &Path, lock_path: &Path) -> Result<Fetcher, String> {
        std::fs::create_dir_all(cache_dir).map_err(|e| e.to_string())?;
        let lock = match std::fs::read_to_string(lock_path) {
            Ok(t) => toml::from_str(&t).map_err(|e| format!("lock: {e}"))?,
            Err(_) => Lock::default(),
        };
        Ok(Fetcher { cache_dir: cache_dir.to_path_buf(), lock_path: lock_path.to_path_buf(), lock })
    }

    /// Default locations: data/cache and data/catalog.lock.toml under the repo root,
    /// overridable with ESIM_CACHE.
    pub fn default_for(repo_root: &Path) -> Result<Fetcher, String> {
        let cache = std::env::var("ESIM_CACHE").map(PathBuf::from).unwrap_or_else(|_| repo_root.join("data/cache"));
        Fetcher::new(&cache, &repo_root.join("data/catalog.lock.toml"))
    }

    /// Return a local path for `url`, downloading if absent. If the lock knows the file,
    /// the bytes must match its sha256; if not, the hash is recorded.
    pub fn fetch(&mut self, key: &str, url: &str) -> Result<PathBuf, String> {
        let name = key.replace(['/', ' '], "_");
        let path = self.cache_dir.join(&name);
        if !path.exists() {
            let tmp = self.cache_dir.join(format!("{name}.part"));
            let resp = ureq::get(url).call().map_err(|e| format!("GET {url}: {e}"))?;
            let mut reader = resp.into_reader();
            let mut f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
            let mut buf = vec![0u8; 1 << 20];
            loop {
                let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
                if n == 0 { break; }
                f.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            }
            drop(f);
            std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        }
        let (sha, bytes) = sha256_file(&path)?;
        match self.lock.files.get(key) {
            Some(e) if e.sha256 != sha => return Err(format!("{key}: sha256 {sha} does not match lock {}", e.sha256)),
            Some(_) => {}
            None => {
                self.lock.files.insert(key.to_string(), Entry { url: url.to_string(), sha256: sha, bytes, fetched: today() });
                self.save_lock()?;
            }
        }
        Ok(path)
    }

    /// Fetch `url`, hash the raw bytes into the lock, but keep only `reduce(raw)` in the
    /// cache: for sources whose answer is large and mostly structure (a 2 MB JSON for
    /// 6,000 numbers). The lock records the raw bytes a run used; the cache holds what the
    /// reader needs. Retries politely on 429 and 5xx.
    pub fn fetch_reduced(&mut self, key: &str, url: &str, reduce: &dyn Fn(&[u8]) -> Result<Vec<u8>, String>) -> Result<PathBuf, String> {
        let name = key.replace(['/', ' '], "_");
        let path = self.cache_dir.join(&name);
        if path.exists() { return Ok(path); }
        let mut raw = Vec::new();
        let mut wait = 2u64;
        for attempt in 0..6 {
            match ureq::get(url).call() {
                Ok(resp) => { resp.into_reader().read_to_end(&mut raw).map_err(|e| e.to_string())?; break; }
                Err(ureq::Error::Status(code, _)) if (code == 429 || code >= 500) && attempt < 5 => {
                    eprintln!("fetch: {url}: {code}; waiting {wait}s"); std::thread::sleep(std::time::Duration::from_secs(wait)); wait *= 2;
                }
                Err(e) => return Err(format!("GET {url}: {e}")),
            }
        }
        if raw.is_empty() { return Err(format!("GET {url}: no bytes")); }
        let sha = format!("{:x}", Sha256::digest(&raw));
        match self.lock.files.get(key) {
            Some(e) if e.sha256 != sha => return Err(format!("{key}: sha256 {sha} does not match lock {}", e.sha256)),
            Some(_) => {}
            None => { self.lock.files.insert(key.to_string(), Entry { url: url.to_string(), sha256: sha, bytes: raw.len() as u64, fetched: today() }); self.save_lock()?; }
        }
        let reduced = reduce(&raw)?;
        let tmp = self.cache_dir.join(format!("{name}.part"));
        std::fs::write(&tmp, &reduced).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        Ok(path)
    }


    /// Fetch one byte range of `url` (a GRIB record by its index), hash the raw bytes into the
    /// lock under `key`, keep only `reduce(raw)` in the cache. The lock's url carries the range.
    pub fn fetch_range_reduced(&mut self, key: &str, url: &str, start: u64, end: u64, reduce: &dyn Fn(&[u8]) -> Result<Vec<u8>, String>) -> Result<PathBuf, String> {
        let name = key.replace(['/', ' '], "_");
        let path = self.cache_dir.join(&name);
        if path.exists() { return Ok(path); }
        let ranged = format!("{url}#bytes={start}-{end}");
        let mut raw = Vec::new();
        let mut wait = 2u64;
        for attempt in 0..6 {
            match ureq::get(url).set("Range", &format!("bytes={start}-{end}")).call() {
                Ok(resp) => { resp.into_reader().read_to_end(&mut raw).map_err(|e| e.to_string())?; break; }
                Err(ureq::Error::Status(code, _)) if (code == 429 || code >= 500) && attempt < 5 => {
                    eprintln!("fetch: {ranged}: {code}; waiting {wait}s"); std::thread::sleep(std::time::Duration::from_secs(wait)); wait *= 2;
                }
                Err(e) => return Err(format!("GET {ranged}: {e}")),
            }
        }
        if raw.is_empty() { return Err(format!("GET {ranged}: no bytes")); }
        let sha = format!("{:x}", Sha256::digest(&raw));
        match self.lock.files.get(key) {
            Some(e) if e.sha256 != sha => return Err(format!("{key}: sha256 {sha} does not match lock {}", e.sha256)),
            Some(_) => {}
            None => { self.lock.files.insert(key.to_string(), Entry { url: ranged, sha256: sha, bytes: raw.len() as u64, fetched: today() }); self.save_lock()?; }
        }
        let reduced = reduce(&raw)?;
        let tmp = self.cache_dir.join(format!("{name}.part"));
        std::fs::write(&tmp, &reduced).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        Ok(path)
    }

    fn save_lock(&self) -> Result<(), String> {
        let text = toml::to_string_pretty(&self.lock).map_err(|e| e.to_string())?;
        std::fs::write(&self.lock_path, format!("# Written by the fetcher. One entry per fetched file: the bytes a run used.\n\n{text}")).map_err(|e| e.to_string())
    }
}

pub fn sha256_file(path: &Path) -> Result<(String, u64), String> {
    let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut total = 0u64;
    loop {
        let n = f.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 { break; }
        h.update(&buf[..n]);
        total += n as u64;
    }
    Ok((format!("{:x}", h.finalize()), total))
}

pub fn today_string() -> String { today() }

fn today() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let days = secs.div_euclid(86_400);
    // civil from days (Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}
