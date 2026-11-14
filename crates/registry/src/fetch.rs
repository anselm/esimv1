//! Fetch by URL into a content cache, verify or record the sha256, and hand back a
//! path. The lock file is the registry's memory of which bytes a run used.
//!
//! Fetching is safe to do from several processes at once and bounded in time: one process at a
//! time fetches a given file (the others wait and then find it), a download is written under a
//! name of its own and renamed into place, the hash list is re-read before it is rewritten, every
//! request has a time limit, and requests to one host are spaced. With `ESIM_OFFLINE=1` nothing is
//! fetched: a missing file is an error that names it. What a process fetched is kept in a tally
//! that `esim prepare` prints.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

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

/// One file this process fetched: its key, its size and how long the request took.
#[derive(Clone, Debug)]
pub struct Fetched { pub key: String, pub bytes: u64, pub secs: f64 }

static TALLY: Mutex<Vec<Fetched>> = Mutex::new(Vec::new());
static LAST: Mutex<Option<HashMap<String, Instant>>> = Mutex::new(None);

/// What this process has fetched so far.
pub fn fetched() -> Vec<Fetched> { TALLY.lock().map(|t| t.clone()).unwrap_or_default() }
fn tally(key: &str, bytes: u64, t0: Instant) { if let Ok(mut t) = TALLY.lock() { t.push(Fetched { key: key.to_string(), bytes, secs: t0.elapsed().as_secs_f64() }); } }

/// True when fetching is refused (`ESIM_OFFLINE=1`): a run then needs everything on disk already.
pub fn offline() -> bool { std::env::var("ESIM_OFFLINE").map(|v| v == "1" || v == "true").unwrap_or(false) }
fn refuse(key: &str, url: &str) -> String { format!("offline: {key} is not on disk and fetching is refused ({url})") }

/// A client with limits: thirty seconds to connect, two minutes for any one read to make
/// progress, and for an answer (not a large file) a limit on the whole exchange.
pub fn agent(whole_secs: Option<u64>) -> ureq::Agent {
    let b = ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(30)).timeout_read(Duration::from_secs(120));
    match whole_secs { Some(s) => b.timeout(Duration::from_secs(s)).build(), None => b.build() }
}

/// Space requests to one host a fifth of a second apart, so a module that needs hundreds of
/// files does not hammer the server it depends on.
pub fn polite(url: &str) {
    let host = url.split('/').nth(2).unwrap_or("").to_string();
    let wait = { let mut g = LAST.lock().unwrap(); let m = g.get_or_insert_with(HashMap::new); let now = Instant::now(); let gap = Duration::from_millis(200);
        let w = m.get(&host).map(|t| gap.saturating_sub(now.duration_since(*t))).unwrap_or(Duration::ZERO); m.insert(host, now + w); w };
    if !wait.is_zero() { std::thread::sleep(wait); }
}

/// Held while a process fetches one file, or rewrites the hash list: another process asking for
/// the same thing waits here and then finds the work done. Released when dropped, or when the
/// process dies.
pub struct Held(#[allow(dead_code)] std::fs::File);
fn hold(target: &Path) -> Result<Held, String> {
    let dir = target.parent().unwrap_or(Path::new(".")).join(".locks");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let f = std::fs::OpenOptions::new().create(true).write(true).truncate(false).open(dir.join(format!("{}.lock", target.file_name().and_then(|n| n.to_str()).unwrap_or("file")))).map_err(|e| e.to_string())?;
    f.lock().map_err(|e| format!("lock {}: {e}", target.display()))?;
    Ok(Held(f))
}
fn part(path: &Path) -> PathBuf { PathBuf::from(format!("{}.{}.part", path.display(), std::process::id())) }

impl Fetcher {
    pub fn new(cache_dir: &Path, lock_path: &Path) -> Result<Fetcher, String> {
        std::fs::create_dir_all(cache_dir).map_err(|e| e.to_string())?;
        Ok(Fetcher { cache_dir: cache_dir.to_path_buf(), lock_path: lock_path.to_path_buf(), lock: Self::read_lock(lock_path)? })
    }
    fn read_lock(lock_path: &Path) -> Result<Lock, String> {
        match std::fs::read_to_string(lock_path) { Ok(t) => toml::from_str(&t).map_err(|e| format!("lock: {e}")), Err(_) => Ok(Lock::default()) }
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
            if offline() { return Err(refuse(key, url)); }
            let _held = hold(&path)?;
            if !path.exists() {   // nobody fetched it while this process waited
                let t0 = Instant::now();
                let tmp = part(&path);
                polite(url);
                let resp = agent(None).get(url).call().map_err(|e| format!("GET {url}: {e}"))?;
                let mut reader = resp.into_reader();
                let mut f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
                let mut buf = vec![0u8; 1 << 20];
                let mut total = 0u64;
                loop {
                    let n = match reader.read(&mut buf) { Ok(n) => n, Err(e) => { let _ = std::fs::remove_file(&tmp); return Err(format!("GET {url}: {e}")); } };
                    if n == 0 { break; }
                    f.write_all(&buf[..n]).map_err(|e| e.to_string())?; total += n as u64;
                }
                drop(f);
                std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
                tally(key, total, t0);
            }
        }
        let (sha, bytes) = sha256_file(&path)?;
        match self.lock.files.get(key) {
            Some(e) if e.sha256 != sha => return Err(format!("{key}: sha256 {sha} does not match lock {}", e.sha256)),
            Some(_) => {}
            None => self.record(key, Entry { url: url.to_string(), sha256: sha, bytes, fetched: today() })?,
        }
        Ok(path)
    }

    /// The bytes of one answer, with polite retries on 429 and 5xx and a limit on the whole exchange.
    fn answer(url: &str, range: Option<(u64, u64)>, label: &str) -> Result<Vec<u8>, String> {
        let mut raw = Vec::new();
        let mut wait = 2u64;
        for attempt in 0..6 {
            polite(url);
            // a bound on the whole exchange: an Overpass mirror held a connection open for twelve minutes
            // without answering on 2026-11-14, and nothing should wait on one request for ever
            let mut req = agent(Some(300)).get(url);
            if let Some((a, b)) = range { req = req.set("Range", &format!("bytes={a}-{b}")); }
            match req.call() {
                Ok(resp) => { raw.clear(); resp.into_reader().read_to_end(&mut raw).map_err(|e| e.to_string())?; break; }
                Err(ureq::Error::Status(code, _)) if (code == 429 || code >= 500) && attempt < 5 => {
                    eprintln!("fetch: {label}: {code}; waiting {wait}s"); std::thread::sleep(Duration::from_secs(wait)); wait *= 2;
                }
                Err(e) => return Err(format!("GET {label}: {e}")),
            }
        }
        if raw.is_empty() { return Err(format!("GET {label}: no bytes")); }
        Ok(raw)
    }
    /// Fetch an answer, hash the raw bytes into the lock, keep only `reduce(raw)` at `path`.
    fn reduced(&mut self, key: &str, path: PathBuf, url: &str, range: Option<(u64, u64)>, reduce: &dyn Fn(&[u8]) -> Result<Vec<u8>, String>) -> Result<PathBuf, String> {
        if path.exists() { return Ok(path); }
        let label = match range { Some((a, b)) => format!("{url}#bytes={a}-{b}"), None => url.to_string() };
        if offline() { return Err(refuse(key, &label)); }
        if let Some(dir) = path.parent() { std::fs::create_dir_all(dir).map_err(|e| e.to_string())?; }
        let _held = hold(&path)?;
        if path.exists() { return Ok(path); }   // another process fetched it while this one waited
        let t0 = Instant::now();
        let raw = Self::answer(url, range, &label)?;
        let sha = format!("{:x}", Sha256::digest(&raw));
        // the hash list as it is on disk now, not as it was when this process started
        self.lock = Self::read_lock(&self.lock_path)?;
        match self.lock.files.get(key) {
            Some(e) if e.sha256 != sha => return Err(format!("{key}: sha256 {sha} does not match lock {}", e.sha256)),
            Some(_) => {}
            None => self.record(key, Entry { url: label, sha256: sha, bytes: raw.len() as u64, fetched: today() })?,
        }
        let out = reduce(&raw)?;
        let tmp = part(&path);
        std::fs::write(&tmp, &out).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        tally(key, raw.len() as u64, t0);
        Ok(path)
    }

    /// Fetch `url`, hash the raw bytes into the lock, but keep only `reduce(raw)` in the
    /// cache: for sources whose answer is large and mostly structure (a 2 MB JSON for
    /// 6,000 numbers). The lock records the raw bytes a run used; the cache holds what the
    /// reader needs. Retries politely on 429 and 5xx.
    pub fn fetch_reduced(&mut self, key: &str, url: &str, reduce: &dyn Fn(&[u8]) -> Result<Vec<u8>, String>) -> Result<PathBuf, String> {
        let path = self.kept_path(&key.replace(['/', ' '], "_"));
        self.reduced(key, path, url, None, reduce)
    }

    /// Where a reduced answer lives. Most are in the cache, which is not in git and can be
    /// fetched again. An OpenStreetMap answer through Overpass is small once reduced and slow and
    /// unreliable to fetch (ten minutes for one city on 2026-11-14, one mirror hanging, the other
    /// answering 502), so it is kept in `data/osm/`, which is committed: a box that has pulled the
    /// repository never asks Overpass for a window that was fetched once.
    pub fn kept_path(&self, name: &str) -> PathBuf {
        // beside the hash list, in the repository, wherever the cache has been pointed
        if name.starts_with("osm-overpass_") { self.lock_path.parent().unwrap_or(&self.cache_dir).join("osm").join(name) } else { self.cache_dir.join(name) }
    }

    /// Fetch one byte range of `url` (a GRIB record by its index), hash the raw bytes into the
    /// lock under `key`, keep only `reduce(raw)` in the cache. The lock's url carries the range.
    pub fn fetch_range_reduced(&mut self, key: &str, url: &str, start: u64, end: u64, reduce: &dyn Fn(&[u8]) -> Result<Vec<u8>, String>) -> Result<PathBuf, String> {
        let path = self.cache_dir.join(key.replace(['/', ' '], "_"));
        self.reduced(key, path, url, Some((start, end)), reduce)
    }

    /// Add one entry to the hash list on disk: under a hold, re-read the file, add the entry if
    /// it is not there, and replace the file whole. Two processes adding different entries both
    /// keep theirs.
    fn record(&mut self, key: &str, entry: Entry) -> Result<(), String> {
        let _held = hold(&self.lock_path)?;
        let mut on_disk = Self::read_lock(&self.lock_path)?;
        on_disk.files.entry(key.to_string()).or_insert(entry);
        let text = toml::to_string_pretty(&on_disk).map_err(|e| e.to_string())?;
        let tmp = part(&self.lock_path);
        std::fs::write(&tmp, format!("# Written by the fetcher. One entry per fetched file: the bytes a run used.\n\n{text}")).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &self.lock_path).map_err(|e| e.to_string())?;
        self.lock = on_disk;
        Ok(())
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
