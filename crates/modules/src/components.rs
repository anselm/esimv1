//! `hazard.components`: every physical thing as a component with a place, a fragility, a state and
//! a repair path, driven by the ground module's fields at its cell and the clock, with the networks
//! computed from which components stand. Here: the fuel hub's tanks (the county study's per-tank
//! table, placed on its own coordinates or OpenStreetMap footprints), the Willamette crossings,
//! DOGAMI's transmission structures with the state's chance of failure at each, and DOGAMI's
//! emergency route segments with the state's deformation class at each. Outputs: each tank's
//! release by the study's age rule, the power service per cell from the poles that stand, the
//! reachability of each hospital and the hub along open segments and open crossings, and the
//! crossings' own states. The fragilities are in the event folder's fragility.toml, recalled from
//! Hazus and declared so; the study's totals and the county's statements are the checks.
use crate::quake::{cell_of, phi, root, sample, window_grid};
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Duration, Event, LatLonGrid, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Serialize, Deserialize, Default)]
struct Comp {
    kind: String, id: String, name: String, lat: f64, lon: f64, cell: usize,
    year: i64, prob: f64, state: u8, down: bool, repair_day: i64,
    fill_gal: f64, pct_min: f64, pct_max: f64, river: bool, property: String,
    class: i64, neighbours: Vec<usize>, west: usize, east: usize, people: f64,
}
#[derive(Clone, Serialize, Deserialize, Default)]
struct Curve { medians: Vec<f64>, beta: f64, gf_median: f64, gf_beta: f64, restore: Vec<i64>, fail_state: u8 }

fn tf(t: &toml::Value, k: &str) -> f64 { t.get(k).and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64))).unwrap_or(0.0) }
fn ts(t: &toml::Value, k: &str) -> String { t.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string() }
fn tarr(t: &toml::Value, k: &str) -> Vec<f64> { t.get(k).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_float().or_else(|| x.as_integer().map(|i| i as f64))).collect()).unwrap_or_default() }

#[derive(Serialize, Deserialize)]
pub struct Components {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String, soils: String, coarse_arcsec: f64,
    force: Vec<String>, storage_cut: f64, retrofit: bool,
    tank: Curve, bridge: Curve, crews_per_day: f64, served_poles: usize, route_days: Vec<(f64, f64)>, post_1993_max: f64,
    comps: Vec<Comp>,
    hospitals: Vec<(String, f64, f64)>, hub: (f64, f64), hub_seg: usize, hosp_seg: HashMap<String, usize>,
    coarse: Option<LatLonGrid>, coarse_people: Vec<f32>, coarse_poles: Vec<Vec<usize>>,
    fired: bool, scheduled: bool, day: i64,
    release_gal: f64, release_river_gal: f64, poles_failed: i64, poles_repaired: i64,
    #[serde(skip)] grid: Option<LatLonGrid>,
}

impl Components {
    pub fn new(name: &str, p: &toml::Value) -> Result<Components, String> {
        let force: Vec<String> = p.get("force").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        let mut o = Components {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 45.43), lat_n: param_f64(p, "lat_n", 45.66), lon_w: param_f64(p, "lon_w", -122.85), lon_e: param_f64(p, "lon_e", -122.47),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "portland-cascadia").to_string(),
            soils: param_str(p, "soils", "wet").to_string(),
            coarse_arcsec: param_f64(p, "coarse_arcsec", 10.0),
            force, storage_cut: param_f64(p, "storage_cut", 0.0), retrofit: p.get("retrofit").and_then(|v| v.as_bool()).unwrap_or(false),
            tank: Curve::default(), bridge: Curve::default(), crews_per_day: 12.0, served_poles: 2, route_days: Vec::new(), post_1993_max: 0.1,
            comps: Vec::new(), hospitals: Vec::new(), hub: (0.0, 0.0), hub_seg: 0, hosp_seg: HashMap::new(),
            coarse: None, coarse_people: Vec::new(), coarse_poles: Vec::new(),
            fired: false, scheduled: false, day: 0, release_gal: 0.0, release_river_gal: 0.0, poles_failed: 0, poles_repaired: 0,
            grid: None,
        };
        o.load()?;
        Ok(o)
    }
    fn table(folder: &str, file: &str) -> Result<toml::Value, String> {
        let path = root().join("data/events").join(folder).join(file);
        std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?.parse::<toml::Value>().map_err(|e| format!("{file}: {e}"))
    }
    fn csv(folder: &str, file: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        let path = root().join("data/events").join(folder).join(file);
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut lines = text.lines();
        let head: Vec<String> = lines.next().ok_or("empty csv")?.split(',').map(|s| s.trim().to_string()).collect();
        Ok((head, lines.filter(|l| !l.trim().is_empty()).map(|l| l.split(',').map(|s| s.trim().to_string()).collect()).collect()))
    }
    fn load(&mut self) -> Result<(), String> {
        let (g, _) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let fr = Self::table(&self.folder, "fragility.toml")?;
        let curve = |t: &toml::Value| Curve { medians: tarr(t, "medians_g"), beta: tf(t, "beta"), gf_median: tf(t, "gf_median_cm"), gf_beta: tf(t, "gf_beta"), restore: tarr(t, "restore_days").iter().map(|&d| d as i64).collect(), fail_state: tf(t, "fail_state").max(tf(t, "closed_state")) as u8 };
        self.tank = curve(fr.get("tank").ok_or("fragility.toml: [tank]")?);
        self.bridge = curve(fr.get("bridge").ok_or("fragility.toml: [bridge]")?);
        self.post_1993_max = tf(fr.get("tank").unwrap(), "post_1993_release_max");
        let pole = fr.get("pole").ok_or("fragility.toml: [pole]")?;
        self.crews_per_day = tf(pole, "crews_per_day"); self.served_poles = tf(pole, "served_poles").max(1.0) as usize;
        self.route_days = fr.get("route").and_then(|r| r.get("closure_days_by_class")).and_then(|v| v.as_array()).map(|a| a.iter().map(|x| { let p = x.as_array().map(|q| q.iter().filter_map(|y| y.as_float().or_else(|| y.as_integer().map(|i| i as f64))).collect::<Vec<_>>()).unwrap_or_default(); (p.first().copied().unwrap_or(0.0), p.get(1).copied().unwrap_or(0.0)) }).collect()).unwrap_or_default();
        let mut comps = Vec::new();
        // tanks: the study's own table
        let (h, rows) = Self::csv(&self.folder, "tanks.csv")?;
        let col = |k: &str| h.iter().position(|x| x == k).ok_or(format!("tanks.csv: no column {k}"));
        let (ci, cn, clat, clon, cfill, cyear, czone, cmin, cmax, cprop, cfail) = (col("id")?, col("contents")?, col("lat")?, col("lon")?, col("fill_gal")?, col("year")?, col("zone")?, col("pct_min")?, col("pct_max")?, col("property")?, col("failure")?);
        for r in &rows {
            let f = |i: usize| r.get(i).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            let (lat, lon) = (f(clat), f(clon));
            if lat == 0.0 || r[cfail].starts_with("No") { continue; }
            comps.push(Comp { kind: "tank".into(), id: r[ci].clone(), name: format!("{} {}", r[cprop], r[ci]), lat, lon, cell: cell_of(&g, lat, lon), year: r.get(cyear).and_then(|s| s.parse().ok()).unwrap_or(0), prob: f64::NAN, fill_gal: f(cfill), pct_min: f(cmin) / 100.0, pct_max: f(cmax) / 100.0, river: r[czone].contains("Water"), property: r[cprop].clone(), repair_day: -1, ..Default::default() });
            let _ = cn;
        }
        // crossings
        let br = Self::table(&self.folder, "bridges.toml")?;
        for b in br.get("bridge").and_then(|v| v.as_array()).cloned().unwrap_or_default() {
            let (lat, lon) = (tf(&b, "lat"), tf(&b, "lon"));
            comps.push(Comp { kind: "bridge".into(), id: ts(&b, "name"), name: ts(&b, "name"), lat, lon, cell: cell_of(&g, lat, lon), year: tf(&b, "opened") as i64, prob: f64::NAN, repair_day: -1, ..Default::default() });
        }
        // poles: DOGAMI's structures with the state's chance of failure at each
        let (h, rows) = Self::csv(&self.folder, "poles.csv")?;
        let (cx, cy, cid, cp) = (h.iter().position(|x| x == "X").ok_or("poles.csv: X")?, h.iter().position(|x| x == "Y").ok_or("poles.csv: Y")?, h.iter().position(|x| x == "PoleID").ok_or("poles.csv: PoleID")?, h.iter().position(|x| x == if self.soils == "wet" { "CSZ_wet_Prob" } else { "CSZ_dry_Prob" }).ok_or("poles.csv: prob")?);
        for r in &rows {
            let (lon, lat) = (r[cx].parse::<f64>().unwrap_or(0.0), r[cy].parse::<f64>().unwrap_or(0.0));
            comps.push(Comp { kind: "pole".into(), id: r[cid].clone(), name: format!("pole {}", r[cid]), lat, lon, cell: cell_of(&g, lat, lon), prob: r[cp].parse::<f64>().unwrap_or(0.0) / 100.0, repair_day: -1, ..Default::default() });
        }
        // route segments and the graph
        let rt = Self::table(&self.folder, "routes.toml")?;
        let segs = rt.get("segment").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let base = comps.len();
        let mut seg_index: HashMap<i64, usize> = HashMap::new();
        for (k, s) in segs.iter().enumerate() { seg_index.insert(tf(s, "id") as i64, base + k); }
        for s in &segs {
            let (lat, lon) = (tf(s, "lat"), tf(s, "lon"));
            let nb: Vec<usize> = s.get("neighbours").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_integer()).filter_map(|i| seg_index.get(&i).copied()).collect()).unwrap_or_default();
            comps.push(Comp { kind: "route".into(), id: format!("{}", tf(s, "id") as i64), name: format!("segment {}", tf(s, "id") as i64), lat, lon, cell: cell_of(&g, lat, lon), class: tf(s, if self.soils == "wet" { "class" } else { "class_dry" }) as i64, neighbours: nb, prob: f64::NAN, repair_day: -1, people: tf(s, "length_m"), ..Default::default() });
        }
        for l in rt.get("link").and_then(|v| v.as_array()).cloned().unwrap_or_default() {
            let name = ts(&l, "bridge");
            let (w, e) = (seg_index.get(&(tf(&l, "west") as i64)).copied().unwrap_or(usize::MAX), seg_index.get(&(tf(&l, "east") as i64)).copied().unwrap_or(usize::MAX));
            if let Some(b) = comps.iter_mut().find(|c| c.kind == "bridge" && c.name == name) { b.west = w; b.east = e; }
        }
        if let Some(n) = rt.get("nearest") {
            self.hub_seg = seg_index.get(&(tf(n, "hub") as i64)).copied().unwrap_or(usize::MAX);
            if let Some(t) = n.as_table() { for (k, v) in t { if k != "hub" { if let Some(i) = v.as_integer().and_then(|i| seg_index.get(&i)) { self.hosp_seg.insert(k.clone(), *i); } } } }
        }
        let ho = Self::table(&self.folder, "hospitals.toml")?;
        self.hospitals = ho.get("hospital").and_then(|v| v.as_array()).map(|a| a.iter().map(|h| (ts(h, "name"), tf(h, "lat"), tf(h, "lon"))).collect()).unwrap_or_default();
        let hb = Self::table(&self.folder, "hub.toml")?; let hb = hb.get("hub").ok_or("hub.toml: [hub]")?; self.hub = (tf(hb, "lat"), tf(hb, "lon"));
        // the coarse grid for power service: people per cell and the nearest poles
        let step = (self.coarse_arcsec / self.res_arcsec).round().max(1.0) as usize;
        let cg = LatLonGrid { nx: (g.nx + step - 1) / step, ny: (g.ny + step - 1) / step, dlon: g.dlon * step as f64, dlat: g.dlat * step as f64, lon_w: g.lon_w, lat_n: g.lat_n };
        let res = sample(&self.folder, "people_resident.tif", &g)?;
        let mut people = vec![0.0f32; cg.nx * cg.ny];
        for r in 0..g.ny { for c in 0..g.nx { let v = res[r * g.nx + c]; if !v.is_nan() { people[(r / step) * cg.nx + c / step] += v; } } }
        let poles: Vec<(usize, f64, f64)> = comps.iter().enumerate().filter(|(_, c)| c.kind == "pole").map(|(i, c)| (i, c.lat, c.lon)).collect();
        let k = self.served_poles;
        let mut nearest = Vec::with_capacity(cg.nx * cg.ny);
        for r in 0..cg.ny { for c in 0..cg.nx {
            let (lat, lon) = (cg.lat_n - (r as f64 + 0.5) * cg.dlat, cg.lon_w + (c as f64 + 0.5) * cg.dlon);
            let mut best: Vec<(f64, usize)> = poles.iter().map(|&(i, plat, plon)| (((plat - lat) * 111.0).powi(2) + ((plon - lon) * 78.0).powi(2), i)).collect();
            best.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            nearest.push(best.iter().take(k).map(|x| x.1).collect::<Vec<_>>());
        } }
        for (ci, ps) in nearest.iter().enumerate() { for &pi in ps { comps[pi].people += people[ci] as f64; } }
        self.coarse = Some(cg); self.coarse_people = people; self.coarse_poles = nearest;
        self.comps = comps; self.grid = Some(g);
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "portland-cascadia");
        let fr = Self::table(folder, "fragility.toml").ok();
        let mut d = Describe::new("hazard.components", name, "Every physical thing as a component with a place, a fragility, a state and a repair path: the fuel hub's tanks from the county study's own per-tank table (621 rows, placed on the study's coordinates where it gives them, else on OpenStreetMap tank footprints), the Willamette crossings, DOGAMI's transmission structures with the state's chance of failure at each (6,146 in the window), and DOGAMI's emergency route segments with the state's deformation class at each (137). On the rupture each reads the ground at its cell and draws its state; tanks release by the study's age rule (pre-1993: its per-tank 50 to 100 percent, later: up to 10 percent), to the river where the study says so; crews repair poles at a stated rate in order of the people served; segments reopen by class; crossings by damage state. Computed and published daily: the power service per coarse cell and the people-weighted share, each hospital's reachability from the hub along open segments and open crossings, the crossings' states. The fragilities are recalled from Hazus and declared so in fragility.toml; the study's release total (94.6 to 193.7 million gallons) and the county's statement on its bridges are the checks, reported.")
            .priority(1).dt("1 day after the rupture")
            .param("folder", folder, "", "the event folder with tanks.csv, poles.csv, routes.toml, bridges.toml, hospitals.toml, hub.toml and fragility.toml")
            .param("soils", param_str(p, "soils", "wet"), "", "which of DOGAMI's columns the poles and routes read: wet or dry")
            .param("coarse_arcsec", param_f64(p, "coarse_arcsec", 10.0), "arcsec", "the power service grid")
            .param("storage_cut", param_f64(p, "storage_cut", 0.0), "", "fraction of every tank's contents removed before the day (Ordinance 2026-312's fifth, applied to the stock)")
            .param("retrofit", p.get("retrofit").and_then(|v| v.as_bool()).unwrap_or(false), "", "every tank releases as one built after 1993: up to post_1993_release_max of its contents")
            .param("force", json!(p.get("force").cloned().map(|v| serde_json::to_value(v).unwrap_or(json!([]))).unwrap_or(json!([]))), "", "preconditions: 'kind:id:state' sets a component's damage state regardless of the draw (tank:KMW128:4, bridge:Burnside Bridge:0)")
            .subscribe("quake.rupture", "the instant")
            .read_on("quake.pga_g", "g", true, "quake.rupture", "at each component").read_on("quake.liq_p", "1", true, "quake.rupture", "").read_on("quake.liq_pgd_cm", "cm", true, "quake.rupture", "").read_on("quake.ls_p", "1", true, "quake.rupture", "").read_on("quake.ls_pgd_cm", "cm", true, "quake.rupture", "")
            .publish("component.state", "{ kind, id, name, lat, lon, state, down, repair_day, day } at the cell: every tank, crossing and segment on the rupture, failed poles, and each repair")
            .publish("tank.release", "{ id, name, gal, river } at the cell, per failed tank").publish("hub.release", "{ gal, river_gal, tanks_failed } once, the sum, for the fuel ledger's owner")
            .publish("network.state", "{ day, power_share, routes_open_share, bridges_open, reachable: { hospital: bool }, hospital_power: { hospital: bool }, any_reachable } each day, computed from the components that stand")
            .frame("power_up", "1").frame("tank_release_gal", "gal")
            .report("power.share", "1").report("power.poles_failed", "poles").report("power.poles_repaired", "poles").report("routes.open_share", "1").report("bridges.open", "crossings").report("tanks.failed", "tanks").report("tanks.release_gal", "gal").report("tanks.release_river_gal", "gal").report("fit.release_vs_study_min", "1").report("fit.release_vs_study_max", "1")
            .asset(&format!("events/{folder}"), "the component tables")
            .view("map", "power service", "power_up", "1 where the nearest poles stand, by coarse cell; the people-weighted share is the counter")
            .view("map", "fuel released, by tank", "tank_release_gal", "gallons leaving each tank on the rupture, by coarse cell")
            .view("counter", "share of people with power", "power.share", "computed from the poles, not a ramp")
            .view("counter", "poles failed", "power.poles_failed", "of 6,146 in the window, by DOGAMI's chance at each")
            .view("counter", "crossings open", "bridges.open", "by each crossing's own fragility and repair time")
            .view("counter", "emergency routes open", "routes.open_share", "share of segment length")
            .view("counter", "tanks failed", "tanks.failed", "of the study's active tanks")
            .view("counter", "fuel released", "tanks.release_gal", "the sum over tanks; the study's range is 94.6 to 193.7 million")
            .view("chart", "share of people with power", "power.share", "the computed curve; the Resilience Plan's one to three months is the check")
            .view("chart", "emergency routes open", "routes.open_share", "")
            .view("chart", "crossings open", "bridges.open", "")
            .view("text", "what this does not compute", "The poles carry no network: a cell has power when its nearest poles stand, which stands in for the lines, substations and the grid's own switching that DOGAMI's bundle does not carry. The route segments are DOGAMI's deformation classes turned into days by guessed ranges; debris from the buildings is not on them. The tanks' shaking fragility is recalled; their ground failure uses the buildings' fitted deformation median. The water system is not here at all.", "");
        if let Some(fr) = fr { d = d.table("fragility", json!(fr)); }
        d
    }
    fn draw_state(&self, c: &Curve, pga: f32, liq_p: f32, liq_pgd: f32, ls_p: f32, ls_pgd: f32, u: f64) -> u8 {
        if pga.is_nan() || pga <= 0.0 || c.medians.len() < 4 { return 0; }
        let a = pga as f64;
        let ex: Vec<f64> = c.medians.iter().map(|m| phi((a / m).ln() / c.beta)).collect();
        let gf = |p: f32, pgd: f32| -> f64 { if p.is_nan() || pgd.is_nan() || p <= 0.0 || pgd <= 0.0 || c.gf_median <= 0.0 { 0.0 } else { p as f64 * phi((pgd as f64 / c.gf_median).ln() / c.gf_beta) } };
        let pg = 1.0 - (1.0 - gf(liq_p, liq_pgd)) * (1.0 - gf(ls_p, ls_pgd));
        // ground failure takes any state to complete; otherwise the shaking's state by the draw
        if u < pg { return 4; }
        let v = (u - pg) / (1.0 - pg).max(1e-9);
        let mut s = 0u8; for (k, e) in ex.iter().enumerate() { if v < *e { s = k as u8 + 1; } }
        s
    }
    fn network(&self) -> (f64, f64, Vec<String>, HashMap<String, bool>, HashMap<String, bool>) {
        let cg = self.coarse.as_ref().unwrap();
        let (mut up, mut tot) = (0.0, 0.0);
        let mut up_cells = vec![false; cg.nx * cg.ny];
        for (ci, ps) in self.coarse_poles.iter().enumerate() { let ok = ps.iter().all(|&pi| !self.comps[pi].down); up_cells[ci] = ok; let pp = self.coarse_people[ci] as f64; tot += pp; if ok { up += pp; } }
        let hospital_power: HashMap<String, bool> = self.hospitals.iter().map(|(n, lat, lon)| (n.clone(), up_cells[cell_of(cg, *lat, *lon)])).collect();
        let open: Vec<String> = self.comps.iter().filter(|c| c.kind == "bridge" && !c.down).map(|c| c.name.clone()).collect();
        let (mut len_open, mut len_tot) = (0.0, 0.0);
        for c in self.comps.iter().filter(|c| c.kind == "route") { len_tot += c.people; if !c.down { len_open += c.people; } }
        // reachability from the hub's segment over open segments and open crossings
        let mut seen: HashMap<usize, bool> = HashMap::new();
        if self.hub_seg != usize::MAX && !self.comps[self.hub_seg].down {
            let mut q = VecDeque::new(); q.push_back(self.hub_seg); seen.insert(self.hub_seg, true);
            while let Some(i) = q.pop_front() {
                let nb: Vec<usize> = self.comps[i].neighbours.clone();
                let mut next: Vec<usize> = nb.into_iter().filter(|&j| !self.comps[j].down).collect();
                for b in self.comps.iter().filter(|c| c.kind == "bridge" && !c.down) { if b.west == i && b.east != usize::MAX && !self.comps[b.east].down { next.push(b.east); } if b.east == i && b.west != usize::MAX && !self.comps[b.west].down { next.push(b.west); } }
                for j in next { if !seen.contains_key(&j) { seen.insert(j, true); q.push_back(j); } }
            }
        }
        let reachable: HashMap<String, bool> = self.hospitals.iter().map(|(n, _, _)| (n.clone(), self.hosp_seg.get(n).map(|s| seen.contains_key(s)).unwrap_or(false))).collect();
        (if tot > 0.0 { up / tot } else { 1.0 }, if len_tot > 0.0 { len_open / len_tot } else { 1.0 }, open, reachable, hospital_power)
    }
    fn publish_state(&self, ctx: &mut Ctx) {
        let (power, routes, open, reachable, hp) = self.network();
        let any = reachable.values().any(|&b| b);
        ctx.publish("network.state", json!({ "day": self.day, "power_share": power, "routes_open_share": routes, "bridges_open": open, "reachable": reachable, "hospital_power": hp, "any_reachable": any }));
        ctx.report("power.share", "1", power); ctx.report("routes.open_share", "1", routes); ctx.report("bridges.open", "crossings", open.len() as f64);
        ctx.report("power.poles_failed", "poles", self.poles_failed as f64); ctx.report("power.poles_repaired", "poles", self.poles_repaired as f64);
        ctx.report("tanks.failed", "tanks", self.comps.iter().filter(|c| c.kind == "tank" && c.down).count() as f64);
        ctx.report("tanks.release_gal", "gal", self.release_gal); ctx.report("tanks.release_river_gal", "gal", self.release_river_gal);
        let cg = self.coarse.clone().unwrap();
        let mut up = vec![0.0f32; cg.nx * cg.ny];
        for (ci, ps) in self.coarse_poles.iter().enumerate() { up[ci] = if ps.iter().all(|&pi| !self.comps[pi].down) { 1.0 } else { 0.0 }; }
        ctx.field("power_up", "1", &cg, up);
    }
}

impl Module for Components {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("quake.rupture");
        let cg = self.coarse.clone().unwrap();
        ctx.field("tank_release_gal", "gal", &cg, vec![0.0; cg.nx * cg.ny]);
        self.publish_state(ctx);
        for c in self.comps.iter().filter(|c| c.kind != "pole" && c.kind != "route") { ctx.publish_at(c.cell, "component.state", json!({ "kind": c.kind, "id": c.id, "name": c.name, "lat": c.lat, "lon": c.lon, "state": 0, "down": false, "repair_day": -1, "day": 0 })); }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "quake.rupture" => {
                if self.fired { return; }
                self.fired = true; self.day = 0;
                let g = self.grid.clone().unwrap();
                let n = g.nx * g.ny;
                let take = |name: &str| -> Vec<f32> { ctx.world_read(name).map(|f| f.data.clone()).unwrap_or_else(|| vec![f32::NAN; n]) };
                let (pga, liq_p, liq_pgd, ls_p, ls_pgd) = (take("quake.pga_g"), take("quake.liq_p"), take("quake.liq_pgd_cm"), take("quake.ls_p"), take("quake.ls_pgd_cm"));
                let forced: HashMap<(String, String), u8> = self.force.iter().filter_map(|s| { let p: Vec<&str> = s.splitn(3, ':').collect(); if p.len() == 3 { Some(((p[0].to_string(), p[1].to_string()), p[2].parse().unwrap_or(4))) } else { None } }).collect();
                let cg = self.coarse.clone().unwrap();
                let mut release_map = vec![0.0f32; cg.nx * cg.ny];
                let (tank, bridge, route_days, post_max, cut, retrofit) = (self.tank.clone(), self.bridge.clone(), self.route_days.clone(), self.post_1993_max, self.storage_cut, self.retrofit);
                let mut events: Vec<(usize, serde_json::Value)> = Vec::new();
                let mut releases: Vec<(usize, serde_json::Value)> = Vec::new();
                let (mut rel, mut rel_river, mut failed_poles) = (0.0, 0.0, 0i64);
                for i in 0..self.comps.len() {
                    let u = ctx.rng().f64();
                    let c = &self.comps[i];
                    let at = |v: &Vec<f32>| v.get(c.cell).copied().unwrap_or(f32::NAN);
                    let mut state = match c.kind.as_str() {
                        "tank" => self.draw_state(&tank, at(&pga), at(&liq_p), at(&liq_pgd), at(&ls_p), at(&ls_pgd), u),
                        "bridge" => self.draw_state(&bridge, at(&pga), at(&liq_p), at(&liq_pgd), at(&ls_p), at(&ls_pgd), u),
                        "pole" => if u < c.prob { 4 } else { 0 },
                        "route" => if c.class > 0 { 3 } else { 0 },
                        _ => 0,
                    };
                    if let Some(s) = forced.get(&(c.kind.clone(), c.id.clone())) { state = *s; }
                    let c = &mut self.comps[i];
                    c.state = state;
                    match c.kind.as_str() {
                        "tank" => {
                            c.down = state >= tank.fail_state;
                            if c.down {
                                let v = ctx.rng().f64();
                                let frac = if !retrofit && (c.year == 0 || c.year < 1993) { c.pct_min + (c.pct_max - c.pct_min) * v } else { post_max * v };
                                let gal = c.fill_gal * (1.0 - cut) * frac;
                                rel += gal; if c.river { rel_river += gal; }
                                let cc = cell_of(&cg, c.lat, c.lon); release_map[cc] += gal as f32;
                                releases.push((c.cell, json!({ "id": c.id, "name": c.name, "gal": gal, "river": c.river, "year": c.year })));
                            }
                        }
                        "bridge" => { c.down = state >= bridge.fail_state; c.repair_day = if c.down { bridge.restore.get((state as usize).saturating_sub(1)).copied().unwrap_or(230) } else { -1 }; }
                        "pole" => { c.down = state >= 4; if c.down { failed_poles += 1; } }
                        "route" => { let (lo, hi) = route_days.get(c.class as usize).copied().unwrap_or((0.0, 0.0)); let v = ctx.rng().f64(); let days = (lo + (hi - lo) * v).round() as i64; c.down = days > 0; c.repair_day = if c.down { days } else { -1 }; }
                        _ => {}
                    }
                    if c.kind != "pole" || c.down { if c.kind != "pole" { events.push((c.cell, json!({ "kind": c.kind, "id": c.id, "name": c.name, "lat": c.lat, "lon": c.lon, "state": c.state, "down": c.down, "repair_day": c.repair_day, "day": 0 }))); } }
                }
                self.poles_failed = failed_poles; self.release_gal = rel; self.release_river_gal = rel_river;
                for (cell, e) in events { ctx.publish_at(cell, "component.state", e); }
                for (cell, e) in releases { ctx.publish_at(cell, "tank.release", e); }
                let failed_tanks = self.comps.iter().filter(|c| c.kind == "tank" && c.down).count();
                ctx.publish("hub.release", json!({ "gal": rel, "river_gal": rel_river, "tanks_failed": failed_tanks }));
                ctx.field("tank_release_gal", "gal", &cg, release_map);
                ctx.report("fit.release_vs_study_min", "1", rel / 94_634_005.0); ctx.report("fit.release_vs_study_max", "1", rel / 193_687_251.0);
                // repair order for the poles: the people served, most first
                self.publish_state(ctx);
                if !self.scheduled { self.scheduled = true; ctx.schedule_with_priority(Duration::days(1), 4, "day", json!(null)); }
            }
            "day" => {
                self.day += 1; let d = self.day;
                // crews: poles in order of the people served
                let mut idx: Vec<usize> = self.comps.iter().enumerate().filter(|(_, c)| c.kind == "pole" && c.down).map(|(i, _)| i).collect();
                idx.sort_by(|a, b| self.comps[*b].people.partial_cmp(&self.comps[*a].people).unwrap());
                for &i in idx.iter().take(self.crews_per_day.round() as usize) { self.comps[i].down = false; self.comps[i].repair_day = d; self.poles_repaired += 1; }
                for i in 0..self.comps.len() {
                    let c = &self.comps[i];
                    if (c.kind == "bridge" || c.kind == "route") && c.down && c.repair_day >= 0 && d >= c.repair_day {
                        let (cell, e) = (c.cell, json!({ "kind": c.kind, "id": c.id, "name": c.name, "lat": c.lat, "lon": c.lon, "state": c.state, "down": false, "repair_day": c.repair_day, "day": d }));
                        self.comps[i].down = false;
                        ctx.publish_at(cell, "component.state", e);
                    }
                }
                self.publish_state(ctx);
                ctx.schedule_with_priority(Duration::days(1), 4, "day", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Components = serde_json::from_value(v).unwrap(); o.grid = window_grid(o.lat_s, o.lat_n, o.lon_w, o.lon_e, o.res_arcsec).ok().map(|x| x.0); *self = o; }
}
