//! P1 Planet: the bottom of the cascade, zero-dimensional. Four kinds that together make
//! the first planetary rung: observed emissions enter a carbon ledger; an impulse-response
//! carbon cycle decides how much stays in the air; a two-layer energy balance turns the
//! extra CO2 into a radiative forcing and warms a surface layer and a deep ocean; and an
//! observations module reports the real records (Mauna Loa, Law Dome, HadCRUT5, GISTEMP)
//! as series so the run can be scored against them with `esim score`.
//!
//! Nothing here is spatial. The honest unit of this rung is the planet as one box, and
//! the point is to show that observed emissions, mass conservation and two well known
//! response functions reproduce the measured CO2 and temperature curves without being
//! tuned to them. What this rung leaves out is said in each kind's describe.
//!
//! Carbon ledger (GtC): fossil_reserve -> atmosphere; land_use -> atmosphere;
//! atmosphere -> sinks. Stocks: atmosphere, sinks (both held by the carbon.pools module).
//! Energy ledger (J): space -> surface (forcing); surface -> space (radiative response);
//! surface -> deep (ocean heat uptake). Stocks: surface_heat, deep_heat.

use esim_kernel::describe::Describe;
use esim_kernel::manifest::param_f64;
use esim_kernel::{Ctx, Duration, Event, Module, Time};
use esim_registry::planet::{self, Emissions};
use esim_registry::{Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const CARBON: &str = "carbon";
pub const ENERGY: &str = "energy";
const SECS_PER_YEAR: f64 = 365.25 * 86_400.0;
/// Earth's surface area, m2 (sphere of the registry's mean radius).
const EARTH_AREA_M2: f64 = 4.0 * std::f64::consts::PI * esim_registry::EARTH_RADIUS_M * esim_registry::EARTH_RADIUS_M;

fn root() -> PathBuf { std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")) }

fn asset_url(cat: &Catalog, id: &str) -> Result<String, String> {
    Ok(cat.asset(id).ok_or_else(|| format!("catalog has no {id}"))?.url_template.clone())
}

fn year_month(t: Time) -> (i64, i64) {
    let s = t.to_iso();
    (s[0..4].parse().unwrap(), s[5..7].parse().unwrap())
}

/// Seconds from `now` to the first instant of the next calendar month.
fn to_next_month(now: Time) -> Duration {
    let (y, m) = year_month(now);
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    Time::parse(&format!("{ny:04}-{nm:02}-01")).unwrap().since(now)
}

/// Seconds from `now` to the next 1 July strictly after now.
fn to_next_midyear(now: Time) -> Duration {
    let (y, _) = year_month(now);
    let mut t = Time::parse(&format!("{y:04}-07-01")).unwrap();
    if t <= now { t = Time::parse(&format!("{:04}-07-01", y + 1)).unwrap(); }
    t.since(now)
}

/// Fraction of a year between two instants (for monthly integration steps).
fn years_between(a: Time, b: Time) -> f64 { b.since(a).as_secs_f64() / SECS_PER_YEAR }

// ---------------------------------------------------------------------------------------
// carbon.emissions: observed global emissions into the atmosphere, month by month.

#[derive(Serialize, Deserialize)]
pub struct EmissionsIn {
    name: String,
    include_land_use: bool,
    hold_last: bool,
    /// What to assume after the record ends: "hold" (the last year's rate), "stop", or
    /// "zero_by:YYYY" (a straight line from the last recorded rate to zero in that year).
    after_record: String,
    last_tick: Option<Time>,
    fossil_total: f64, luc_total: f64,
    #[serde(skip)] data: Emissions,
}

impl EmissionsIn {
    pub fn new(name: &str, p: &toml::Value) -> Result<EmissionsIn, String> {
        let mut e = EmissionsIn {
            name: name.to_string(),
            include_land_use: p.get("include_land_use").and_then(|v| v.as_bool()).unwrap_or(true),
            hold_last: p.get("hold_last").and_then(|v| v.as_bool()).unwrap_or(true),
            after_record: esim_kernel::manifest::param_str(p, "after_record", "hold").to_string(),
            last_tick: None, fossil_total: 0.0, luc_total: 0.0, data: Emissions::default(),
        };
        e.load()?;
        Ok(e)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = root();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let mut f = Fetcher::default_for(&root)?;
        self.data = planet::owid_world_emissions(&mut f, &asset_url(&cat, "owid-co2-data")?)?;
        Ok(())
    }
    /// Annual rate for a year; beyond the record, the stated assumption. This is the one
    /// place a planetary run stops being a hindcast, and the manifest must say which.
    fn rate(&self, s: &BTreeMap<i64, f64>, y: i64) -> f64 {
        if let Some(v) = s.get(&y) { return *v; }
        let Some((&ly, &lv)) = s.iter().next_back() else { return 0.0 };
        if y < ly { return 0.0; } // before the record: nothing (the start concentration carries it)
        if !self.hold_last { return 0.0; }
        match self.after_record.split_once(':') {
            Some(("zero_by", yr)) => {
                let target: f64 = yr.parse().unwrap_or(ly as f64);
                if target <= ly as f64 { 0.0 } else { (lv * (1.0 - (y as f64 - ly as f64) / (target - ly as f64))).max(0.0) }
            }
            _ if self.after_record == "stop" => 0.0,
            _ => lv,
        }
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        Describe::new("carbon.emissions", name, "Observed global CO2 emissions (Global Carbon Project via Our World in Data) released into the atmosphere month by month, fossil and land-use change as separate ledger sources. Nothing is modelled; this is the forcing that the record says happened.")
            .priority(0).dt("1 month (calendar)")
            .param("include_land_use", p.get("include_land_use").and_then(|v| v.as_bool()).unwrap_or(true), "", "land-use-change CO2 (deforestation etc.) as a second source account")
            .param("hold_last", p.get("hold_last").and_then(|v| v.as_bool()).unwrap_or(true), "", "false stops emissions when the record ends, whatever after_record says")
            .param("after_record", esim_kernel::manifest::param_str(p, "after_record", "hold"), "", "assumption after the record's last year (2024): hold = the last rate for good; stop; zero_by:YYYY = a straight line to zero in that year. An assumption, never a forecast; the manifest notes must say which was chosen and why")
            .publish("carbon.emitted", "{ gtc, fossil_gtc, luc_gtc } for the month just added")
            .ledger(CARBON, "GtC", 1e-6)
            .report("carbon.emissions_gtc_yr", "GtC/yr").report("carbon.cumulative_gtc", "GtC")
            .asset("owid-co2-data", "World rows: co2 and land_use_change_co2, Mt CO2 per year")
            .note("Mt CO2 to GtC by 12.011/44.009. The annual total is spread evenly over the twelve months of the year.")
    }
}

impl Module for EmissionsIn {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(CARBON, "GtC", 1e-6);
        self.last_tick = Some(ctx.now());
        ctx.schedule_with_priority(to_next_month(ctx.now()), 0, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        let prev = self.last_tick.unwrap_or(ctx.now());
        let (y, _) = year_month(prev); // the month just completed belongs to its own year
        let dt_yr = years_between(prev, ctx.now());
        let fossil = self.rate(&self.data.fossil_gtc, y) * dt_yr;
        let luc = if self.include_land_use { self.rate(&self.data.land_use_gtc, y) * dt_yr } else { 0.0 };
        ctx.transfer(CARBON, "fossil_reserve", "atmosphere", fossil);
        ctx.transfer(CARBON, "land_use", "atmosphere", luc);
        self.fossil_total += fossil; self.luc_total += luc;
        ctx.publish("carbon.emitted", json!({ "gtc": fossil + luc, "fossil_gtc": fossil, "luc_gtc": luc }));
        ctx.report("carbon.emissions_gtc_yr", "GtC/yr", (fossil + luc) / dt_yr);
        ctx.report("carbon.cumulative_gtc", "GtC", self.fossil_total + self.luc_total);
        self.last_tick = Some(ctx.now());
        ctx.schedule_with_priority(to_next_month(ctx.now()), 0, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut e: EmissionsIn = serde_json::from_value(v).unwrap(); e.load().expect("emissions reload"); *self = e; }
}

// ---------------------------------------------------------------------------------------
// carbon.pools: the airborne fraction by impulse response.

#[derive(Serialize, Deserialize)]
pub struct Pools {
    name: String,
    co2_start_ppm: f64, gtc_per_ppm: f64,
    a: [f64; 4], tau: [f64; 3],
    pools: [f64; 4],
    sinks: f64, emitted: f64,
    last_tick: Option<Time>,
}

impl Pools {
    pub fn new(name: &str, p: &toml::Value) -> Result<Pools, String> {
        let s = Self::settings(p);
        Ok(Pools { name: name.to_string(), co2_start_ppm: s.0, gtc_per_ppm: s.1, a: s.2, tau: s.3, pools: [0.0; 4], sinks: 0.0, emitted: 0.0, last_tick: None })
    }
    fn settings(p: &toml::Value) -> (f64, f64, [f64; 4], [f64; 3]) {
        (
            param_f64(p, "co2_start_ppm", 286.8), param_f64(p, "gtc_per_ppm", 2.124),
            [param_f64(p, "a0", 0.2173), param_f64(p, "a1", 0.2240), param_f64(p, "a2", 0.2824), param_f64(p, "a3", 0.2763)],
            [param_f64(p, "tau1_yr", 394.4), param_f64(p, "tau2_yr", 36.54), param_f64(p, "tau3_yr", 4.304)],
        )
    }
    fn excess(&self) -> f64 { self.pools.iter().sum() }
    fn atmosphere(&self) -> f64 { self.co2_start_ppm * self.gtc_per_ppm + self.excess() }
    fn ppm(&self) -> f64 { self.atmosphere() / self.gtc_per_ppm }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let s = Self::settings(p);
        Describe::new("carbon.pools", name, "Atmospheric CO2 as a mass ledger: emissions arrive as events, a fixed fraction stays for good and three fractions decay to the land-and-ocean sinks on three timescales (the Joos et al. 2013 multi-model impulse response). Reports the airborne concentration in ppm. Linear: sink saturation and climate-carbon feedback are not represented.")
            .priority(1).dt("1 month (calendar)")
            .param("co2_start_ppm", s.0, "ppm", "Law Dome spline at 1850 (MacFarling Meure et al. 2006): 286.8")
            .param("gtc_per_ppm", s.1, "GtC/ppm", "2.124, the conversion used by the Global Carbon Budget")
            .param("a0", s.2[0], "", "fraction never removed on these timescales; Joos et al. 2013 multi-model mean as tabulated in IPCC AR5 8.SM, recalled from memory and to be checked against the paper")
            .param("a1", s.2[1], "", "fraction on tau1").param("a2", s.2[2], "", "fraction on tau2").param("a3", s.2[3], "", "fraction on tau3")
            .param("tau1_yr", s.3[0], "yr", "").param("tau2_yr", s.3[1], "yr", "").param("tau3_yr", s.3[2], "yr", "")
            .subscribe("carbon.emitted", "{ gtc } added to the four pools by their fractions")
            .publish("carbon.co2", "{ ppm } after each monthly step")
            .ledger(CARBON, "GtC", 1e-6)
            .report("carbon.co2_ppm", "ppm").report("carbon.excess_gtc", "GtC").report("carbon.airborne_fraction", "")
            .note("The ledger makes the turnover-versus-excess distinction concrete: molecules exchange fast, the excess mass drains slowly, and the stock the ledger checks is the excess.")
    }
}

impl Module for Pools {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(CARBON, "GtC", 1e-6);
        ctx.declare_stock(CARBON, "atmosphere", self.atmosphere());
        ctx.declare_stock(CARBON, "sinks", self.sinks);
        ctx.subscribe("carbon.emitted");
        self.last_tick = Some(ctx.now());
        ctx.report("carbon.co2_ppm", "ppm", self.ppm());
        ctx.schedule_with_priority(to_next_month(ctx.now()), 1, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "carbon.emitted" => {
                let g = ev.data.get("gtc").and_then(|v| v.as_f64()).unwrap_or(0.0);
                for i in 0..4 { self.pools[i] += g * self.a[i]; }
                self.emitted += g;
                ctx.declare_stock(CARBON, "atmosphere", self.atmosphere());
            }
            "tick" => {
                let dt_yr = years_between(self.last_tick.unwrap_or(ctx.now()), ctx.now());
                let mut removed = 0.0;
                for i in 1..4 {
                    let left = self.pools[i] * (-dt_yr / self.tau[i - 1]).exp();
                    removed += self.pools[i] - left;
                    self.pools[i] = left;
                }
                self.sinks += removed;
                ctx.transfer(CARBON, "atmosphere", "sinks", removed);
                ctx.declare_stock(CARBON, "atmosphere", self.atmosphere());
                ctx.declare_stock(CARBON, "sinks", self.sinks);
                let ppm = self.ppm();
                ctx.publish("carbon.co2", json!({ "ppm": ppm }));
                ctx.report("carbon.co2_ppm", "ppm", ppm);
                ctx.report("carbon.excess_gtc", "GtC", self.excess());
                if self.emitted > 0.0 { ctx.report("carbon.airborne_fraction", "", self.excess() / self.emitted); }
                self.last_tick = Some(ctx.now());
                ctx.schedule_with_priority(to_next_month(ctx.now()), 1, "tick", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}

// ---------------------------------------------------------------------------------------
// energy.twolayer: forcing from CO2, a surface layer and a deep ocean.

#[derive(Serialize, Deserialize)]
pub struct TwoLayer {
    name: String,
    co2_ref_ppm: f64, alpha: f64, lambda: f64, gamma: f64, c_surface: f64, c_deep: f64, extra_forcing: f64,
    /// forcings other modules publish on `forcing.extra`, by name, W/m2
    extra: BTreeMap<String, f64>,
    ppm: f64,
    /// stored heat relative to the start, J
    h_surface: f64, h_deep: f64,
    last_tick: Option<Time>,
}

impl TwoLayer {
    pub fn new(name: &str, p: &toml::Value) -> Result<TwoLayer, String> {
        let co2_ref_ppm = param_f64(p, "co2_ref_ppm", 286.8);
        Ok(TwoLayer {
            name: name.to_string(), co2_ref_ppm,
            alpha: param_f64(p, "alpha_w_m2", 5.35), lambda: param_f64(p, "lambda_w_m2_k", 1.13), gamma: param_f64(p, "gamma_w_m2_k", 0.73),
            c_surface: param_f64(p, "c_surface_w_yr_m2_k", 7.3), c_deep: param_f64(p, "c_deep_w_yr_m2_k", 91.0),
            extra_forcing: param_f64(p, "extra_forcing_w_m2", 0.0),
            extra: BTreeMap::new(),
            ppm: co2_ref_ppm, h_surface: 0.0, h_deep: 0.0, last_tick: None,
        })
    }
    /// Heat capacities in J/K for the whole planet's area.
    fn cap_surface(&self) -> f64 { self.c_surface * SECS_PER_YEAR * EARTH_AREA_M2 }
    fn cap_deep(&self) -> f64 { self.c_deep * SECS_PER_YEAR * EARTH_AREA_M2 }
    fn t_surface(&self) -> f64 { self.h_surface / self.cap_surface() }
    fn t_deep(&self) -> f64 { self.h_deep / self.cap_deep() }
    fn forcing(&self) -> f64 { self.alpha * (self.ppm / self.co2_ref_ppm).ln() + self.extra_forcing + self.extra.values().sum::<f64>() }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        Describe::new("energy.twolayer", name, "Global energy balance in two layers (Held et al. 2010; Geoffroy et al. 2013): CO2 forcing 5.35 ln(C/C0) warms a surface layer that radiates in proportion to its anomaly and exchanges heat with a deep ocean. Energy ledger in joules: forcing in from space, response out to space, uptake into the deep. CO2 only; other greenhouse gases and aerosols enter only through extra_forcing_w_m2.")
            .priority(2).dt("1 month (calendar)")
            .param("co2_ref_ppm", param_f64(p, "co2_ref_ppm", 286.8), "ppm", "concentration at which forcing is zero; the carbon module's start")
            .param("alpha_w_m2", param_f64(p, "alpha_w_m2", 5.35), "W/m2", "Myhre et al. 1998 simplified expression for CO2")
            .param("lambda_w_m2_k", param_f64(p, "lambda_w_m2_k", 1.13), "W/m2/K", "climate feedback parameter; Geoffroy et al. 2013 CMIP5 multi-model mean, recalled from memory and to be checked")
            .param("gamma_w_m2_k", param_f64(p, "gamma_w_m2_k", 0.73), "W/m2/K", "deep ocean heat exchange coefficient; same source and caveat")
            .param("c_surface_w_yr_m2_k", param_f64(p, "c_surface_w_yr_m2_k", 7.3), "W yr/m2/K", "surface layer heat capacity; same source and caveat")
            .param("c_deep_w_yr_m2_k", param_f64(p, "c_deep_w_yr_m2_k", 91.0), "W yr/m2/K", "deep ocean heat capacity; same source and caveat")
            .param("extra_forcing_w_m2", param_f64(p, "extra_forcing_w_m2", 0.0), "W/m2", "constant non-CO2 forcing; zero means CO2 only")
            .subscribe("carbon.co2", "{ ppm } sets the forcing for the next step")
            .subscribe("forcing.extra", "{ name, w_m2 } from other modules (methane, N2O, aerosols), summed by name")
            .publish("energy.state", "{ gmst_c, deep_c, forcing_w_m2, imbalance_w_m2, stored_zj } after each step, for modules downstream of the climate")
            .ledger(ENERGY, "J", 1e10)
            .report("energy.gmst_anom_c", "degC").report("energy.deep_anom_c", "degC").report("energy.forcing_w_m2", "W/m2")
            .report("energy.imbalance_w_m2", "W/m2").report("energy.stored_zj", "ZJ")
            .note("Explicit monthly steps; the fastest timescale is C/(lambda+gamma), about four years, so the step is well resolved.")
    }
}

impl Module for TwoLayer {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(ENERGY, "J", 1e10);
        ctx.declare_stock(ENERGY, "surface_heat", self.h_surface);
        ctx.declare_stock(ENERGY, "deep_heat", self.h_deep);
        ctx.subscribe("carbon.co2");
        ctx.subscribe("forcing.extra");
        self.last_tick = Some(ctx.now());
        ctx.schedule_with_priority(to_next_month(ctx.now()), 2, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "carbon.co2" => { if let Some(p) = ev.data.get("ppm").and_then(|v| v.as_f64()) { self.ppm = p; } }
            "forcing.extra" => {
                if let (Some(n), Some(w)) = (ev.data.get("name").and_then(|v| v.as_str()), ev.data.get("w_m2").and_then(|v| v.as_f64())) { self.extra.insert(n.to_string(), w); }
            }
            "tick" => {
                let dt = ctx.now().since(self.last_tick.unwrap_or(ctx.now())).as_secs_f64();
                let (ts, td) = (self.t_surface(), self.t_deep());
                let f = self.forcing();
                let in_j = f * EARTH_AREA_M2 * dt;
                let out_j = self.lambda * ts * EARTH_AREA_M2 * dt;
                let down_j = self.gamma * (ts - td) * EARTH_AREA_M2 * dt;
                self.h_surface += in_j - out_j - down_j;
                self.h_deep += down_j;
                ctx.transfer(ENERGY, "space", "surface_heat", in_j);
                ctx.transfer(ENERGY, "surface_heat", "space", out_j);
                ctx.transfer(ENERGY, "surface_heat", "deep_heat", down_j);
                ctx.declare_stock(ENERGY, "surface_heat", self.h_surface);
                ctx.declare_stock(ENERGY, "deep_heat", self.h_deep);
                ctx.report("energy.gmst_anom_c", "degC", self.t_surface());
                ctx.report("energy.deep_anom_c", "degC", self.t_deep());
                ctx.report("energy.forcing_w_m2", "W/m2", f);
                ctx.report("energy.imbalance_w_m2", "W/m2", (in_j - out_j) / (EARTH_AREA_M2 * dt));
                ctx.report("energy.stored_zj", "ZJ", (self.h_surface + self.h_deep) / 1e21);
                for (n, w) in &self.extra { ctx.report(&format!("energy.forcing_{n}_w_m2"), "W/m2", *w); }
                ctx.publish("energy.state", json!({ "gmst_c": self.t_surface(), "deep_c": self.t_deep(), "forcing_w_m2": f,
                    "imbalance_w_m2": (in_j - out_j) / (EARTH_AREA_M2 * dt), "stored_zj": (self.h_surface + self.h_deep) / 1e21 }));
                self.last_tick = Some(ctx.now());
                ctx.schedule_with_priority(to_next_month(ctx.now()), 2, "tick", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}

// ---------------------------------------------------------------------------------------
// carbon.gases: methane and nitrous oxide from the records, as forcing for the energy module.

#[derive(Serialize, Deserialize)]
pub struct Gases {
    name: String,
    ch4_0: f64, n2o_0: f64,
    #[serde(skip)] ch4: BTreeMap<i64, f64>,
    #[serde(skip)] n2o: BTreeMap<i64, f64>,
}

/// The CH4 and N2O overlap term of the simplified expressions (IPCC TAR Table 6.2, after
/// Myhre et al. 1998). M and N in ppb.
fn overlap(m: f64, n: f64) -> f64 { 0.47 * (1.0 + 2.01e-5 * (m * n).powf(0.75) + 5.31e-15 * m * (m * n).powf(1.52)).ln() }

impl Gases {
    pub fn new(name: &str, p: &toml::Value) -> Result<Gases, String> {
        let mut g = Gases { name: name.to_string(), ch4_0: param_f64(p, "ch4_preindustrial_ppb", 722.0), n2o_0: param_f64(p, "n2o_preindustrial_ppb", 270.0), ch4: BTreeMap::new(), n2o: BTreeMap::new() };
        g.load()?;
        Ok(g)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = root();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let mut f = Fetcher::default_for(&root)?;
        let law = asset_url(&cat, "law-dome-2006")?;
        // ice core until the instrument records begin, then NOAA GML
        self.ch4 = planet::law_dome_ch4(&mut f, &law)?;
        let gml_ch4 = planet::gml_annual(&mut f, "noaa-gml/ch4_annmean_gl.csv", &asset_url(&cat, "noaa-gml-ch4-annmean-gl")?)?;
        let first = *gml_ch4.keys().next().unwrap(); self.ch4.retain(|y, _| *y < first); self.ch4.extend(gml_ch4);
        self.n2o = planet::law_dome_n2o(&mut f, &law)?;
        let gml_n2o = planet::gml_annual(&mut f, "noaa-gml/n2o_annmean_gl.csv", &asset_url(&cat, "noaa-gml-n2o-annmean-gl")?)?;
        let first = *gml_n2o.keys().next().unwrap(); self.n2o.retain(|y, _| *y < first); self.n2o.extend(gml_n2o);
        Ok(())
    }
    /// Value for a year: the record, else the last recorded value (held), else the first.
    fn at(s: &BTreeMap<i64, f64>, y: i64) -> f64 {
        if let Some(v) = s.get(&y) { return *v; }
        match (s.range(..y).next_back(), s.range(y..).next()) { (Some((_, v)), _) => *v, (None, Some((_, v))) => *v, _ => 0.0 }
    }
    fn forcings(&self, y: i64) -> (f64, f64, f64, f64) {
        let (m, n) = (Self::at(&self.ch4, y), Self::at(&self.n2o, y));
        let (m0, n0) = (self.ch4_0, self.n2o_0);
        let f_ch4 = 0.036 * (m.sqrt() - m0.sqrt()) - (overlap(m, n0) - overlap(m0, n0));
        let f_n2o = 0.12 * (n.sqrt() - n0.sqrt()) - (overlap(m0, n) - overlap(m0, n0));
        (m, n, f_ch4, f_n2o)
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        Describe::new("carbon.gases", name, "Methane and nitrous oxide concentrations from the records (Law Dome ice core until the NOAA global means begin in 1984 and 2001), turned into radiative forcing by the simplified expressions of Myhre et al. 1998 with the CH4-N2O overlap, and published to the energy module. Beyond the record the last value is held: an assumption. No emissions and no chemistry; concentrations are read, not modelled.")
            .priority(1).dt("1 year (1 January)")
            .param("ch4_preindustrial_ppb", param_f64(p, "ch4_preindustrial_ppb", 722.0), "ppb", "IPCC AR5 value, recalled")
            .param("n2o_preindustrial_ppb", param_f64(p, "n2o_preindustrial_ppb", 270.0), "ppb", "IPCC AR5 value, recalled")
            .publish("forcing.extra", "{ name: methane | n2o, w_m2 } each January")
            .report("gases.ch4_ppb", "ppb").report("gases.n2o_ppb", "ppb").report("gases.forcing_ch4_w_m2", "W/m2").report("gases.forcing_n2o_w_m2", "W/m2")
            .asset("law-dome-2006", "CH4 and N2O splines before the instrument records").asset("noaa-gml-ch4-annmean-gl", "global annual mean CH4 from 1984").asset("noaa-gml-n2o-annmean-gl", "global annual mean N2O from 2001")
            .note("The expressions are IPCC TAR Table 6.2 (Myhre et al. 1998); Etminan et al. 2016 revised methane upward by about a quarter and is the next step. Aerosols are still absent.")
    }
}

impl Module for Gases {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        self.emit(ctx);
        ctx.schedule_with_priority(to_next_year(ctx.now()), 1, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        self.emit(ctx);
        ctx.schedule_with_priority(to_next_year(ctx.now()), 1, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut g: Gases = serde_json::from_value(v).unwrap(); g.load().expect("gases reload"); *self = g; }
}

impl Gases {
    fn emit(&self, ctx: &mut Ctx) {
        let (y, _) = year_month(ctx.now());
        let (m, n, f_ch4, f_n2o) = self.forcings(y);
        ctx.publish("forcing.extra", json!({ "name": "methane", "w_m2": f_ch4 }));
        ctx.publish("forcing.extra", json!({ "name": "n2o", "w_m2": f_n2o }));
        ctx.report("gases.ch4_ppb", "ppb", m); ctx.report("gases.n2o_ppb", "ppb", n);
        ctx.report("gases.forcing_ch4_w_m2", "W/m2", f_ch4); ctx.report("gases.forcing_n2o_w_m2", "W/m2", f_n2o);
    }
}

/// Seconds from `now` to the next 1 January strictly after now.
fn to_next_year(now: Time) -> Duration {
    let (y, _) = year_month(now);
    Time::parse(&format!("{:04}-01-01", y + 1)).unwrap().since(now)
}

// ---------------------------------------------------------------------------------------
// sea.thermal: sea level from the heat the energy ledger has stored.

#[derive(Serialize, Deserialize)]
pub struct SeaThermal {
    name: String,
    m_per_yj: f64,
    last_m: f64,
}

impl SeaThermal {
    pub fn new(name: &str, p: &toml::Value) -> Result<SeaThermal, String> {
        Ok(SeaThermal { name: name.to_string(), m_per_yj: param_f64(p, "m_per_yj", 0.11), last_m: 0.0 })
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        Describe::new("sea.thermal", name, "Global mean sea level rise from thermal expansion alone: the energy module's stored heat times an expansion efficiency. Ice sheets and glaciers, which are the larger term this century, are not here yet; the number is a floor, and the describe says so.")
            .priority(3).dt("follows energy.state")
            .param("m_per_yj", param_f64(p, "m_per_yj", 0.11), "m/YJ", "expansion efficiency of ocean heat uptake, about 0.11 m per 10^24 J (Kuhlbrodt and Gregory 2012, recalled; to be checked)")
            .subscribe("energy.state", "{ stored_zj }")
            .report("sea.thermal_m", "m")
            .note("Thermal expansion only. No scoreboard yet: a satellite altimetry series has not been fetched.")
    }
}

impl Module for SeaThermal {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) { ctx.subscribe("energy.state"); }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "energy.state" { return; }
        let Some(zj) = ev.data.get("stored_zj").and_then(|v| v.as_f64()) else { return };
        self.last_m = zj / 1000.0 * self.m_per_yj;
        ctx.report("sea.thermal_m", "m", self.last_m);
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}

// ---------------------------------------------------------------------------------------
// obs.planet: the records, reported mid-year so the run can be scored.

#[derive(Serialize, Deserialize)]
pub struct PlanetObs {
    name: String,
    baseline_from: i64, baseline_to: i64,
    #[serde(skip)] co2: BTreeMap<i64, f64>,
    #[serde(skip)] hadcrut: BTreeMap<i64, f64>,
    #[serde(skip)] gistemp: BTreeMap<i64, f64>,
    hadcrut_base: f64, gistemp_base: f64,
}

impl PlanetObs {
    pub fn new(name: &str, p: &toml::Value) -> Result<PlanetObs, String> {
        let mut o = PlanetObs {
            name: name.to_string(),
            baseline_from: param_f64(p, "baseline_from", 1850.0) as i64, baseline_to: param_f64(p, "baseline_to", 1900.0) as i64,
            co2: BTreeMap::new(), hadcrut: BTreeMap::new(), gistemp: BTreeMap::new(), hadcrut_base: 0.0, gistemp_base: 0.0,
        };
        o.load()?;
        Ok(o)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = root();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let mut f = Fetcher::default_for(&root)?;
        // Law Dome before Mauna Loa, Mauna Loa from 1959: one CO2 series.
        self.co2 = planet::law_dome_co2(&mut f, &asset_url(&cat, "law-dome-2006")?)?;
        let mlo = planet::mauna_loa_annual(&mut f, &asset_url(&cat, "noaa-gml-co2-annmean-mlo")?)?;
        let first = *mlo.keys().next().unwrap();
        self.co2.retain(|y, _| *y < first);
        self.co2.extend(mlo);
        self.hadcrut = planet::hadcrut5_annual(&mut f, &asset_url(&cat, "hadcrut5-global-annual")?)?;
        self.gistemp = planet::gistemp_annual(&mut f, &asset_url(&cat, "gistemp-global-annual")?)?;
        self.hadcrut_base = planet::window_mean(&self.hadcrut, self.baseline_from, self.baseline_to).unwrap_or(0.0);
        self.gistemp_base = planet::window_mean(&self.gistemp, self.baseline_from, self.baseline_to).unwrap_or(0.0);
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        Describe::new("obs.planet", name, "The planetary scoreboards as series: CO2 from the Law Dome ice-core spline until Mauna Loa begins in 1959, then Mauna Loa annual means; global mean surface temperature from HadCRUT5 and GISTEMP, each shifted to a zero mean over the baseline years so they compare with a model that starts at zero. Reported on 1 July of each year.")
            .priority(3).dt("1 year (1 July)")
            .param("baseline_from", param_f64(p, "baseline_from", 1850.0), "yr", "temperature anomalies are re-zeroed over this window").param("baseline_to", param_f64(p, "baseline_to", 1900.0), "yr", "GISTEMP starts in 1880, so its baseline is the overlap")
            .report("obs.co2_ppm", "ppm").report("obs.gmst_hadcrut5_c", "degC").report("obs.gmst_gistemp_c", "degC")
            .asset("law-dome-2006", "CO2 spline, years before Mauna Loa").asset("noaa-gml-co2-annmean-mlo", "annual mean CO2 from 1959")
            .asset("hadcrut5-global-annual", "annual anomaly vs 1961-1990").asset("gistemp-global-annual", "annual anomaly vs 1951-1980, J-D column")
            .note("A series reported here is an observation, never a model output; score a model series against it with esim score.")
    }
}

impl Module for PlanetObs {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.schedule_with_priority(to_next_midyear(ctx.now()), 3, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        let (y, _) = year_month(ctx.now());
        if let Some(v) = self.co2.get(&y) { ctx.report("obs.co2_ppm", "ppm", *v); }
        if let Some(v) = self.hadcrut.get(&y) { ctx.report("obs.gmst_hadcrut5_c", "degC", v - self.hadcrut_base); }
        if let Some(v) = self.gistemp.get(&y) { ctx.report("obs.gmst_gistemp_c", "degC", v - self.gistemp_base); }
        ctx.schedule_with_priority(to_next_midyear(ctx.now()), 3, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: PlanetObs = serde_json::from_value(v).unwrap(); o.load().expect("obs reload"); *self = o; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn month_steps_land_on_firsts() {
        let t = Time::parse("1850-01-01").unwrap();
        let n = t.add(to_next_month(t));
        assert_eq!(n.to_iso(), "1850-02-01T00:00:00Z");
        let d = Time::parse("1899-12-01").unwrap();
        assert_eq!(d.add(to_next_month(d)).to_iso(), "1900-01-01T00:00:00Z");
        assert_eq!(t.add(to_next_midyear(t)).to_iso(), "1850-07-01T00:00:00Z");
        let j = Time::parse("1850-07-01").unwrap();
        assert_eq!(j.add(to_next_midyear(j)).to_iso(), "1851-07-01T00:00:00Z");
    }
}
