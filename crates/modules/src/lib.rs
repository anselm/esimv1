//! Modules. Each is built by `factory` from its manifest `kind`.

pub mod water;

use esim_kernel::Module;

pub fn factory(name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String> {
    let kind = params.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    build(kind, name, params)
}

/// Build by kind. The kernel passes params only, so the CLI folds `kind` into them.
pub fn build(kind: &str, name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String> {
    match kind {
        "water.source" => Ok(Box::new(water::Source::new(name, params)?)),
        "water.bucket" => Ok(Box::new(water::Bucket::new(name, params)?)),
        other => Err(format!("unknown module kind {other}")),
    }
}

pub fn kinds() -> &'static [&'static str] { &["water.source", "water.bucket"] }
