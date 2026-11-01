//! M0 proof modules: a source that rains into a bucket, and buckets that drain into the
//! next account at their own rate. Two rates, one ledger, and it must close. A bucket
//! with `leak` set misreports its stock on purpose, so the test suite can prove the
//! ledger catches it.

use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;

const LEDGER: &str = "water";

/// Pours `rate` cubic metres per second from the `from` boundary account into the
/// `into` module's account, every `dt`. Owns no stock.
#[derive(Serialize, Deserialize)]
pub struct Source {
    name: String,
    rate: f64,
    dt: Duration,
    from: String,
    into: String,
    poured: f64,
}

impl Source {
    pub fn new(name: &str, p: &toml::Value) -> Result<Source, String> {
        Ok(Source {
            name: name.to_string(),
            rate: param_f64(p, "rate_m3_s", 1.0),
            dt: param_duration(p, "dt", "1h")?,
            from: param_str(p, "from", "atmosphere").to_string(),
            into: param_str(p, "into", "bucket").to_string(),
            poured: 0.0,
        })
    }
}

impl Module for Source {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(LEDGER, "m3", 1e-6);
        ctx.schedule(self.dt, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        let amount = self.rate * self.dt.as_secs_f64();
        self.poured += amount;
        ctx.transfer(LEDGER, &self.from, &self.into, amount);
        ctx.send(&self.into, "inflow", json!({ "m3": amount }));
        ctx.report(&format!("{}.poured_m3", self.name), "m3", self.poured);
        ctx.schedule(self.dt, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}

/// Holds water. Receives `inflow` events addressed to it, and every `dt` releases a
/// fraction `k` per second of its volume to the `outlet` account (a module's account or
/// a boundary), transferring on the ledger and notifying the outlet if it is a module.
#[derive(Serialize, Deserialize)]
pub struct Bucket {
    name: String,
    volume: f64,
    k: f64,
    dt: Duration,
    outlet: String,
    outlet_is_module: bool,
    /// Fraction of each release the bucket forgets to account for. Zero unless testing.
    leak: f64,
}

impl Bucket {
    pub fn new(name: &str, p: &toml::Value) -> Result<Bucket, String> {
        Ok(Bucket {
            name: name.to_string(),
            volume: param_f64(p, "volume_m3", 0.0),
            k: param_f64(p, "k_per_s", 1e-5),
            dt: param_duration(p, "dt", "6h")?,
            outlet: param_str(p, "outlet", "sea").to_string(),
            outlet_is_module: p.get("outlet_is_module").and_then(|v| v.as_bool()).unwrap_or(false),
            leak: param_f64(p, "leak", 0.0),
        })
    }
}

impl Module for Bucket {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(LEDGER, "m3", 1e-6);
        ctx.declare_stock(LEDGER, &self.name, self.volume);
        ctx.schedule(self.dt, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "inflow" => {
                let m3 = ev.data["m3"].as_f64().unwrap_or(0.0);
                self.volume += m3;
                // The sender already transferred on the ledger; we only declare.
                ctx.declare_stock(LEDGER, &self.name, self.volume);
            }
            "tick" => {
                let frac = 1.0 - (-self.k * self.dt.as_secs_f64()).exp();
                let out = self.volume * frac;
                self.volume -= out * (1.0 + self.leak); // a leaky bucket lies about itself
                let me = self.name.clone();
                ctx.transfer(LEDGER, &me, &self.outlet, out);
                if self.outlet_is_module {
                    ctx.send(&self.outlet, "inflow", json!({ "m3": out }));
                }
                ctx.declare_stock(LEDGER, &me, self.volume);
                ctx.report(&format!("{}.volume_m3", me), "m3", self.volume);
                ctx.report(&format!("{}.outflow_m3_s", me), "m3/s", out / self.dt.as_secs_f64());
                ctx.schedule(self.dt, "tick", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}
