//! M2: surface temperature as a second field at its own rate, driven by the sun's
//! incident energy through the shared world store. Seam test 1 of the brief: two
//! fields, two clocks, one energy ledger that must close across both modules.
//!
//! Physics, deliberately simple and labelled: each cell is a slab of heat capacity C
//! (J per m2 per K). It absorbs (1 - albedo) of the incident shortwave, reflects the
//! rest to space, and radiates emissivity * sigma * (T^4 - T_sky^4) to a fixed sky
//! temperature. No air temperature, no turbulent fluxes, no soil conduction; those
//! arrive with weather (M4). The point here is the seam, not the climate.
//!
//! Energy ledger (joules): the sun credits `surface` with incident energy each hour;
//! this module, running faster, drains `surface` into `ground` (absorbed) and
//! `space_reflected`, and radiates `ground` to `space_longwave`. Two stocks are declared:
//! `surface` (what this module has seen but not yet drained) and `ground` (sum of
//! C * (T - 273.15 K) * area). The sun and the heat module therefore have to agree on
//! every joule at every close, or the ledger says so.

use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use serde::{Deserialize, Serialize};
use serde_json::json;

const LEDGER: &str = "energy";
const SIGMA: f64 = 5.670_374_419e-8;
const T0: f64 = 273.15;

#[derive(Serialize, Deserialize)]
pub struct Heat {
    name: String,
    source: String,
    dt: Duration,
    report_every: Duration,
    albedo: f64,
    emissivity: f64,
    heat_capacity: f64,
    t_sky_k: f64,
    t_init_c: f64,
    // state
    t_k: Vec<f64>,
    /// incident joules of the source's current version not yet drained, per cell
    pending: Vec<f64>,
    /// fraction of the source step this module's step represents
    seen_version: u64,
    ground_j: f64,
    pending_j: f64,
    day_sum: Vec<f32>,
    day_steps: u32,
    next_report: Option<Time>,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] areas: Vec<f64>,
}

impl Heat {
    pub fn new(name: &str, p: &toml::Value) -> Result<Heat, String> {
        Ok(Heat {
            name: name.to_string(),
            source: param_str(p, "source", "sun.incident_j").to_string(),
            dt: param_duration(p, "dt", "15m")?,
            report_every: param_duration(p, "report_every", "1d")?,
            albedo: param_f64(p, "albedo", 0.2),
            emissivity: param_f64(p, "emissivity", 0.95),
            heat_capacity: param_f64(p, "heat_capacity_j_m2_k", 3.0e5),
            t_sky_k: param_f64(p, "t_sky_k", 255.0),
            t_init_c: param_f64(p, "t_init_c", 0.0),
            t_k: Vec::new(), pending: Vec::new(), seen_version: 0, ground_j: 0.0, pending_j: 0.0,
            day_sum: Vec::new(), day_steps: 0, next_report: None, grid: None, areas: Vec::new(),
        })
    }

    fn adopt_grid(&mut self, grid: &LatLonGrid) {
        let n = grid.nx * grid.ny;
        self.areas = (0..grid.ny).flat_map(|r| std::iter::repeat_n(grid.row_area(r), grid.nx)).collect();
        if self.t_k.len() != n {
            self.t_k = vec![self.t_init_c + T0; n];
            self.pending = vec![0.0; n];
            self.day_sum = vec![0.0; n];
        }
        self.grid = Some(grid.clone());
    }

    fn ground_stock(&self) -> f64 {
        self.t_k.iter().zip(self.areas.iter()).map(|(&t, &a)| self.heat_capacity * (t - T0) * a).sum()
    }
}

impl Module for Heat {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(LEDGER, "J", 1e6);
        // The grid is the source's; it may not exist until the source's first step, so
        // declare stocks lazily on the first tick. Run after the source at equal times.
        ctx.schedule_with_priority(self.dt, 1, "tick", json!(null));
    }

    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        let Some(src) = ctx.world_read(&self.source) else {
            ctx.schedule_with_priority(self.dt, 1, "tick", json!(null));
            return;
        };
        let (version, data, grid) = (src.version, src.data.clone(), src.grid.clone());
        if self.grid.is_none() {
            self.adopt_grid(&grid);
            self.ground_j = self.ground_stock();
            ctx.declare_stock(LEDGER, "ground", self.ground_j);
            ctx.declare_stock(LEDGER, "surface", 0.0);
            self.next_report = Some(ctx.now().add(self.report_every));
        }
        let dt = self.dt.as_secs_f64();
        // New energy from the source: take it all into pending, exactly as published.
        if version != self.seen_version {
            for (p, &d) in self.pending.iter_mut().zip(data.iter()) { *p += d as f64; }
            self.seen_version = version;
        }
        // Drain pending over the remaining fraction of the source's step. Without
        // knowing the source's dt we drain a fixed fraction per step, set so an hour of
        // incident is consumed in an hour of this module's steps.
        let drain_frac = (dt / 3600.0).min(1.0);
        let (mut absorbed, mut reflected, mut longwave) = (0.0f64, 0.0f64, 0.0f64);
        let mut pending_total = 0.0f64;
        for i in 0..self.t_k.len() {
            let a = self.areas[i];
            let take = self.pending[i] * drain_frac;
            self.pending[i] -= take;
            let abs = take * (1.0 - self.albedo);
            let refl = take - abs;
            let t = self.t_k[i];
            let lw = self.emissivity * SIGMA * (t.powi(4) - self.t_sky_k.powi(4)) * a * dt;
            let net = abs - lw;
            let dt_k = net / (self.heat_capacity * a);
            self.t_k[i] = t + dt_k;
            absorbed += abs; reflected += refl; longwave += lw;
            pending_total += self.pending[i];
            self.day_sum[i] += (self.t_k[i] - T0) as f32;
        }
        self.day_steps += 1;
        ctx.transfer(LEDGER, "surface", "ground", absorbed);
        ctx.transfer(LEDGER, "surface", "space_reflected", reflected);
        ctx.transfer(LEDGER, "ground", "space_longwave", longwave);
        self.ground_j = self.ground_stock();
        self.pending_j = pending_total;
        ctx.declare_stock(LEDGER, "ground", self.ground_j);
        ctx.declare_stock(LEDGER, "surface", self.pending_j);
        let area: f64 = self.areas.iter().sum();
        let mean_c = self.t_k.iter().zip(self.areas.iter()).map(|(&t, &a)| (t - T0) * a).sum::<f64>() / area;
        ctx.report("heat.t_surface_mean_c", "degC", mean_c);
        if let Some(nr) = self.next_report {
            if ctx.now() >= nr {
                let steps = self.day_steps.max(1) as f32;
                let frame: Vec<f32> = self.day_sum.iter().map(|v| v / steps).collect();
                ctx.field("t_surface_daily_mean", "degC", &grid, frame);
                self.day_sum.iter_mut().for_each(|v| *v = 0.0);
                self.day_steps = 0;
                self.next_report = Some(nr.add(self.report_every));
            }
        }
        ctx.schedule_with_priority(self.dt, 1, "tick", json!(null));
    }

    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) {
        let mut h: Heat = serde_json::from_value(v).unwrap();
        // grid and areas are rebuilt from the source field on the next tick
        h.grid = None;
        let keep_t = h.t_k.clone();
        *self = h;
        self.t_k = keep_t;
    }
}

impl Heat {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        let source = param_str(p, "source", "sun.incident_j");
        esim_kernel::describe::Describe::new("heat.slab", name, "A surface heat slab: absorbs the incident energy field, radiates to a sky temperature, keeps the energy ledger closed at a different rate from the sun.")
            .priority(1).dt(param_str(p, "dt", "15m"))
            .param("source", "sun.incident_j", "", "world field of incident energy per cell per source tick")
            .param("dt", "15m", "", "").param("report_every", "1d", "", "")
            .param("albedo", 0.2, "", "").param("emissivity", 0.95, "", "")
            .param("heat_capacity_j_m2_k", 3.0e5, "J/m2/K", "").param("t_sky_k", 255.0, "K", "").param("t_init_c", 0.0, "degC", "")
            .read(source, "J", true, "")
            .ledger("energy", "J", 1e6)
            .frame("t_surface_daily_mean", "degC")
            .report("heat.t_surface_mean_c", "degC")
            .note("Parameter provenance: devlog/20261101-m2-heat.md")
    }
}
