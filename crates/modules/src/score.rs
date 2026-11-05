//! A score per region of the world, in bands, for a glance at the globe. Anselm, 2026-11-05:
//! an era of cheap externalities is ending; regions differ in what they have left, in their
//! ecosystems, air, water, wealth, fairness and institutions; "all of these factors and others
//! constitute a total bottom line ... arguably rankable"; score every region, show it on a
//! globe as a glyph a person reads at a glance, and run it forward so the knock-backs show.
//!
//! This module is the scoring. Nine bands, each a World Bank indicator (anonymous API, the
//! latest value per country), each turned into a 0 to 1 score by percentile rank among the
//! countries that have it, so a band says where a country stands among its peers and never
//! what a good number is. The total is the mean of the bands a country has. That is the
//! starting state, read from the record in the run's first year. From then on the bands move
//! by rules this module declares as placeholders: a country whose people pass the heat
//! thresholds loses living and water; a wild card knocks a band for some years by its kind.
//! Everything that moves a band here is a guess made visible, not a measurement.
use esim_kernel::describe::Describe;
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, Module, Time};
use esim_registry::{countries, worldbank, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The bands, in glyph order from the ground up.
struct Band { name: &'static str, code: &'static str, source: Option<u32>, date: Option<&'static str>, higher_is_better: bool, log: bool, what: &'static str }
const BANDS: [Band; 9] = [
    Band { name: "living", code: "SP.DYN.LE00.IN", source: None, date: None, higher_is_better: true, log: false, what: "life expectancy at birth, years" },
    Band { name: "means", code: "NY.GNP.PCAP.PP.CD", source: None, date: None, higher_is_better: true, log: true, what: "income per person, PPP dollars (ranked on the log)" },
    Band { name: "fairness", code: "SI.POV.GINI", source: None, date: Some("2012:2025"), higher_is_better: false, log: false, what: "Gini index of income, lower is fairer; latest survey since 2012" },
    Band { name: "voice", code: "GOV_WGI_VA.EST", source: Some(3), date: None, higher_is_better: true, log: false, what: "Worldwide Governance Indicators, voice and accountability estimate" },
    Band { name: "air", code: "EN.ATM.PM25.MC.M3", source: None, date: None, higher_is_better: false, log: false, what: "PM2.5 exposure, micrograms per cubic metre, lower is cleaner" },
    Band { name: "forest", code: "AG.LND.FRST.ZS", source: None, date: None, higher_is_better: true, log: false, what: "forest, percent of land" },
    Band { name: "protected", code: "ER.PTD.TOTL.ZS", source: None, date: None, higher_is_better: true, log: false, what: "protected land and sea, percent of territory" },
    Band { name: "water", code: "ER.H2O.INTR.PC", source: None, date: None, higher_is_better: true, log: true, what: "renewable fresh water per person, cubic metres (ranked on the log)" },
    Band { name: "footprint", code: "EN.GHG.CO2.PC.CE.AR5", source: None, date: None, higher_is_better: false, log: false, what: "CO2 per person, tonnes, lower is lighter" },
];

#[derive(Clone, Serialize, Deserialize)]
struct Region { iso3: String, name: String, lat: f64, lon: f64, base: Vec<Option<f64>>, year: Vec<Option<i64>> }

#[derive(Serialize, Deserialize)]
pub struct Score {
    name: String,
    scale: String, every_years: i64, yearly_from: i64,
    heat_living: f64, heat_water: f64, wetbulb_living: f64,
    #[serde(skip)] regions: Vec<Region>,
    /// multipliers on a country's bands from the thresholds it has crossed, by iso3 then band
    crossed: BTreeMap<String, Vec<f64>>,
    /// global knock-downs from wild cards: (band index, multiplier, last year inclusive)
    knocks: Vec<(usize, f64, i64)>,
    last_published: i64,
}

fn root() -> PathBuf { std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")) }
fn year_of(t: Time) -> i64 { t.to_iso()[0..4].parse().unwrap() }
fn to_next_year(now: Time) -> Duration { Time::parse(&format!("{:04}-01-01", year_of(now) + 1)).unwrap().since(now) }
fn scale_asset(scale: &str) -> &'static str { if scale == "50m" { "natural-earth-50m-countries" } else { "natural-earth-110m-countries" } }

/// The centroid of a country's largest ring by area (shoelace), where the glyph stands.
fn centroid(rings: &[Vec<(f64, f64)>]) -> (f64, f64) {
    let mut best = (0.0, 0.0, -1.0);
    for r in rings {
        if r.len() < 3 { continue; }
        let (mut a, mut cx, mut cy) = (0.0, 0.0, 0.0);
        for i in 0..r.len() {
            let (x0, y0) = r[i]; let (x1, y1) = r[(i + 1) % r.len()];
            let w = x0 * y1 - x1 * y0; a += w; cx += (x0 + x1) * w; cy += (y0 + y1) * w;
        }
        if a.abs() < 1e-12 { continue; }
        let area = a.abs() / 2.0;
        if area > best.2 { best = (cx / (3.0 * a), cy / (3.0 * a), area); }
    }
    (best.1, best.0) // (lat, lon)
}

/// Percentile rank in 0..1 (ties averaged), higher value = higher rank; flipped when lower is better.
fn ranks(vals: &[(usize, f64)], higher_is_better: bool) -> Vec<(usize, f64)> {
    let mut v: Vec<(usize, f64)> = vals.to_vec();
    v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let n = v.len();
    let mut out = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let mut j = i; while j + 1 < n && v[j + 1].1 == v[i].1 { j += 1; }
        let r = if n > 1 { (i + j) as f64 / 2.0 / (n - 1) as f64 } else { 0.5 };
        for k in i..=j { out.push((v[k].0, if higher_is_better { r } else { 1.0 - r })); }
        i = j + 1;
    }
    out
}

impl Score {
    pub fn new(name: &str, p: &toml::Value) -> Result<Score, String> {
        let mut m = Score {
            name: name.to_string(), scale: param_str(p, "scale", "50m").to_string(),
            every_years: param_f64(p, "every_years", 5.0) as i64, yearly_from: param_f64(p, "yearly_from", 2020.0) as i64,
            heat_living: param_f64(p, "heat_living", 0.75), heat_water: param_f64(p, "heat_water", 0.75), wetbulb_living: param_f64(p, "wetbulb_living", 0.75),
            regions: Vec::new(), crossed: BTreeMap::new(), knocks: Vec::new(), last_published: -1_000_000,
        };
        m.load()?;
        Ok(m)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = root();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let mut f = Fetcher::default_for(&root)?;
        let url = |id: &str| -> Result<String, String> { Ok(cat.asset(id).ok_or_else(|| format!("catalog has no {id}"))?.url_template.clone()) };
        let cs = countries::natural_earth_countries(&mut f, &url(scale_asset(&self.scale))?)?;
        let wb = url("worldbank-wdi-api")?;
        let mut regions: Vec<Region> = cs.iter().filter(|c| c.iso3.len() == 3 && c.iso3 != "-99").map(|c| {
            let (lat, lon) = centroid(&c.rings);
            Region { iso3: c.iso3.clone(), name: c.name.clone(), lat, lon, base: vec![None; BANDS.len()], year: vec![None; BANDS.len()] }
        }).collect();
        for (b, band) in BANDS.iter().enumerate() {
            let vals = worldbank::latest_by_country(&mut f, &wb, band.code, band.source, band.date)?;
            let mut present = Vec::new();
            for (i, r) in regions.iter_mut().enumerate() {
                if let Some((y, v)) = vals.get(&r.iso3) {
                    let x = if band.log { if *v > 0.0 { v.ln() } else { continue } } else { *v };
                    present.push((i, x)); r.year[b] = Some(*y);
                }
            }
            for (i, s) in ranks(&present, band.higher_is_better) { regions[i].base[b] = Some(s); }
        }
        self.regions = regions;
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let scale = param_str(p, "scale", "50m").to_string();
        Describe::new("score.regions", name, "A score per country in nine bands (living, means, fairness, voice, air, forest, protected, water, footprint), each a World Bank indicator turned into a 0 to 1 percentile rank among the countries that report it, so a band says where a country stands among its peers and never what a good number is; the total is the mean of the bands it has. The starting state is read from the record. From the run's first year the bands move by declared placeholder rules: a country whose people pass the heat thresholds loses part of its living and water bands, and a wild card knocks a band for some years by its kind. Published as a table for the globe view: the glyph at each country's centroid is the bands stacked, the total its height.")
            .priority(8).dt("1 year (1 January); published every every_years, yearly from yearly_from")
            .param("scale", scale.clone(), "", "Natural Earth scale for the country list and centroids")
            .param("every_years", param_f64(p, "every_years", 5.0), "yr", "how often the table is published before yearly_from")
            .param("yearly_from", param_f64(p, "yearly_from", 2020.0), "year", "from this year the table is published every January")
            .param("heat_living", param_f64(p, "heat_living", 0.75), "", "multiplier on the living band once half a country's people live past the climate niche (a guess)")
            .param("heat_water", param_f64(p, "heat_water", 0.75), "", "multiplier on the water band once a country passes the niche (a guess)")
            .param("wetbulb_living", param_f64(p, "wetbulb_living", 0.75), "", "multiplier on the living band once a country passes the 31 °C wet-bulb limit, and again at 35 °C (a guess)")
            .subscribe("people.crossing", "{ iso3, kind } a country past the niche or a wet-bulb limit loses part of its living and water bands for good")
            .subscribe("shock.crossing", "{ kind, forcing_years, emissions_years } a wild card knocks a band for some years by its kind: disease the living band, economy and technology the means, war the living and the voice, an eruption or an impact the air, an ecological card the water and the forest (all guesses, all global)")
            .publish("region.scored", "{ year, bands: [{ name, code, what, higher_is_better }], by_iso3: { ISO3: { name, lat, lon, s: [nine 0..1 scores or null], total } } } every every_years, yearly from yearly_from")
            .report("region.countries_scored", "").report("region.mean_total", "1").report("region.min_total", "1")
            .table("bands", json!(BANDS.iter().map(|b| json!({ "name": b.name, "code": b.code, "what": b.what, "higher_is_better": b.higher_is_better, "ranked_on_log": b.log, "source": if b.source == Some(3) { "World Bank, Worldwide Governance Indicators (API source 3)" } else { "World Bank, World Development Indicators (API v2)" } })).collect::<Vec<_>>()))
            .view("chart", "mean total score", "region.mean_total", "the plain mean over countries, not weighted by people; the globe itself is drawn at the top of the page by the story module")
            .asset("worldbank-wdi-api", "the nine indicators, latest value per country").asset(scale_asset(&scale), "the countries and their centroids")
            .note("The ranks are among the countries that report an indicator, so a country missing a band is not penalised and its total is the mean of what it has. Every rule that moves a band after the first year is a placeholder written on 2026-11-05 and declared in the parameters; the indicators themselves are read, not modelled, and their years differ by band (the table on the run page names each).")
    }
    fn scores(&self, r: &Region, y: i64) -> (Vec<Option<f64>>, Option<f64>) {
        let cross = self.crossed.get(&r.iso3);
        let mut s = Vec::with_capacity(BANDS.len());
        let (mut sum, mut n) = (0.0, 0);
        for b in 0..BANDS.len() {
            let v = r.base[b].map(|v| {
                let mut v = v * cross.map(|c| c[b]).unwrap_or(1.0);
                for (kb, m, until) in &self.knocks { if *kb == b && *until >= y { v *= m; } }
                v.clamp(0.0, 1.0)
            });
            if let Some(v) = v { sum += v; n += 1; }
            s.push(v);
        }
        (s, if n > 0 { Some(sum / n as f64) } else { None })
    }
    fn publish(&mut self, ctx: &mut Ctx) {
        let y = year_of(ctx.now());
        let mut by = serde_json::Map::new();
        let (mut sum, mut n, mut min) = (0.0, 0usize, f64::INFINITY);
        for r in &self.regions {
            let (s, total) = self.scores(r, y);
            if let Some(t) = total { sum += t; n += 1; if t < min { min = t; } }
            by.insert(r.iso3.clone(), json!({ "name": r.name, "lat": (r.lat * 100.0).round() / 100.0, "lon": (r.lon * 100.0).round() / 100.0, "s": s.iter().map(|v| v.map(|x| (x * 1000.0).round() / 1000.0)).collect::<Vec<_>>(), "total": total.map(|t| (t * 1000.0).round() / 1000.0) }));
        }
        ctx.publish("region.scored", json!({ "year": y, "bands": BANDS.iter().map(|b| json!({ "name": b.name, "code": b.code, "what": b.what, "higher_is_better": b.higher_is_better })).collect::<Vec<_>>(), "by_iso3": by }));
        ctx.report("region.countries_scored", "", n as f64);
        if n > 0 { ctx.report("region.mean_total", "1", sum / n as f64); ctx.report("region.min_total", "1", min); }
        self.last_published = y;
    }
}

impl Module for Score {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("people.crossing"); ctx.subscribe("shock.crossing");
        self.publish(ctx);
        ctx.schedule_with_priority(to_next_year(ctx.now()), 8, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "people.crossing" => {
                let Some(iso3) = ev.data["iso3"].as_str() else { return };
                let kind = ev.data["kind"].as_str().unwrap_or("");
                let c = self.crossed.entry(iso3.to_string()).or_insert_with(|| vec![1.0; BANDS.len()]);
                match kind {
                    "niche" => { c[0] *= self.heat_living; c[7] *= self.heat_water; }
                    "wetbulb_a" | "wetbulb_b" => { c[0] *= self.wetbulb_living; }
                    _ => {}
                }
            }
            "shock.crossing" => {
                let y = year_of(ctx.now());
                let kind = ev.data["kind"].as_str().unwrap_or("");
                let yrs = |k: &str, d: f64| ev.data[k].as_f64().unwrap_or(d).ceil() as i64;
                let cooling = ev.data["forcing_w_m2"].as_f64().unwrap_or(0.0) < 0.0;
                match kind {
                    "disease" => self.knocks.push((0, 0.8, y + 2)),
                    "economy" | "solar" | "technology" => self.knocks.push((1, 0.85, y + yrs("emissions_years", 2.0) - 1)),
                    "war" => { self.knocks.push((0, 0.9, y + 4)); self.knocks.push((3, 0.8, y + 4)); }
                    "geology" | "astronomy" if cooling => self.knocks.push((4, 0.7, y + yrs("forcing_years", 1.0) - 1)),
                    "ecology" => { self.knocks.push((7, 0.8, y + 4)); self.knocks.push((5, 0.9, y + 4)); }
                    _ => {}
                }
            }
            "tick" => {
                let y = year_of(ctx.now());
                self.knocks.retain(|(_, _, until)| *until >= y);
                if y >= self.yearly_from || y - self.last_published >= self.every_years { self.publish(ctx); }
                ctx.schedule_with_priority(to_next_year(ctx.now()), 8, "tick", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut m: Score = serde_json::from_value(v).unwrap(); m.load().expect("score reload"); *self = m; }
}
