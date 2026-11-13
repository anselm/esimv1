//! `people.agents`: the people as super-persons, each standing for `scale` people, generated from
//! DOGAMI's own occupant rasters: a home cell drawn by the residents, a work cell drawn by the
//! daytime occupants, a share on electricity-dependent medical devices from the aftermath table.
//! Each day every agent reads the world at its cells: the building damage where it was at the
//! instant decides whether it is dead or displaced (at the buildings' own expected rates, so the
//! totals agree with the cell model), the power service at its home cell whether it has power,
//! the water ramp whether it has water. What this adds to the cell counts is a population that
//! can be followed: the same agent is counted on every day, and a later rung gives it decisions
//! (to leave, to shelter, to travel). The cost is measured and reported: agents per second.
use crate::quake::{root, sample, window_grid};
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Duration, Event, LatLonGrid, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Default)]
struct Agent { home: u32, work: u32, device: bool, dead: bool, in_collapse: bool, displaced: bool, power: bool, water: bool }

#[derive(Serialize, Deserialize)]
pub struct Agents {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String, scale: f64, time_of_day: String, coarse_arcsec: f64,
    agents: Vec<Agent>,
    device_share: f64,
    fired: bool, scheduled: bool, day: i64, water_share: f64,
    #[serde(skip)] grid: Option<LatLonGrid>,
}

fn tf(t: &toml::Value, k: &str) -> f64 { t.get(k).and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64))).unwrap_or(0.0) }

impl Agents {
    pub fn new(name: &str, p: &toml::Value) -> Result<Agents, String> {
        let mut o = Agents {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 45.43), lat_n: param_f64(p, "lat_n", 45.66), lon_w: param_f64(p, "lon_w", -122.85), lon_e: param_f64(p, "lon_e", -122.47),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "portland-cascadia").to_string(),
            scale: param_f64(p, "scale", 10.0), time_of_day: param_str(p, "time_of_day", "day").to_string(), coarse_arcsec: param_f64(p, "coarse_arcsec", 10.0),
            agents: Vec::new(), device_share: 0.0, fired: false, scheduled: false, day: 0, water_share: 1.0, grid: None,
        };
        o.load()?;
        Ok(o)
    }
    fn load(&mut self) -> Result<(), String> {
        let (g, _) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        if let Ok(text) = std::fs::read_to_string(root().join("data/events").join(&self.folder).join("aftermath.toml")) {
            if let Ok(t) = text.parse::<toml::Value>() { if let Some(i) = t.get("inputs") {
                let dme = tf(i, "power_dependent_dme_multnomah") + tf(i, "power_dependent_dme_washington") + tf(i, "power_dependent_dme_clackamas");
                let pop = tf(i, "population_multnomah") + tf(i, "population_washington") + tf(i, "population_clackamas");
                self.device_share = if pop > 0.0 { dme / pop } else { 0.0 };
            } }
        }
        self.grid = Some(g);
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "portland-cascadia");
        Describe::new("people.agents", name, "This module represents the residents as agents, each standing for a fixed number of people set by the scale parameter. It creates them at the start of the run from DOGAMI's maps of where people are. Each agent gets a home cell, drawn in proportion to residents, and a work cell, drawn in proportion to daytime occupants. A share of the agents depend on electrically powered medical devices; the share comes from the emPOWER data of the US Department of Health and Human Services. At the earthquake each agent is at work by day or at home by night. It is placed in a collapsed or a standing building according to the share of buildings that collapsed in that cell, and it dies or is displaced at the rates the buildings module gives for the cell. The agents' death count should agree with the buildings module's count by cell. In the daytime runs of 12 November 2026 it is lower, for a reason not yet found. On each day afterwards an agent has electricity or not according to its home cell, and water according to the schedule for the whole area. The agents make no decisions yet. They are here so that a later version can let them leave, seek shelter or travel, and the module reports how fast it runs so that the cost of that is known.")
            .priority(6).dt("1 day after the rupture")
            .param("folder", folder, "", "the folder of input data for this event, with DOGAMI's maps of residents and of occupants by day and night")
            .param("scale", param_f64(p, "scale", 10.0), "people", "people per agent. 10 gives about 93,000 agents in the modelled area")
            .param("time_of_day", param_str(p, "time_of_day", "day"), "", "where each agent is at the moment of the earthquake: at work by day, at home by night")
            .param("coarse_arcsec", param_f64(p, "coarse_arcsec", 10.0), "arcsec", "the cell size of the electricity service grid the agents read")
            .subscribe("quake.toll", "the buildings module's damage and casualty rates, once the shaking ends").subscribe("lifeline.state", "the day's water share")
            .read_on("quake.cell_collapse_frac", "1", true, "quake.toll", "the share of the cell's buildings that collapsed, as drawn by the buildings module").read_on("quake.cell_death_rate_collapse", "1", true, "quake.toll", "the chance an occupant of a collapsed building dies").read_on("quake.cell_death_rate_other", "1", true, "quake.toll", "the chance an occupant of a standing building dies").read_on("quake.cell_displaced_rate", "1", true, "quake.toll", "expected displaced per resident at each cell").read_on("power.up", "1", false, "lifeline.state", "the components module's electricity service on its coarser grid")
            .publish("agents.toll", "{ dead, dead_in_collapses, displaced } once every agent's outcome is drawn, for the module that keeps the total")
            .report("agents.count", "agents").report("agents.people", "people").report("agents.dead", "people").report("agents.dead_in_collapses", "people").report("agents.in_collapses", "people").report("agents.displaced", "people").report("agents.without_power", "people").report("agents.without_water", "people").report("agents.devices_without_power", "people").report("agents.per_second", "agents/s")
            .frame("agents_without_power", "people")
            .view("counter", "agents", "agents.count", "each agent stands for several people; the next counter gives the people represented")
            .view("counter", "people followed", "agents.people", "agents multiplied by people per agent")
            .view("counter", "dead, by agent", "agents.dead", "Each agent is placed in a collapsed or a standing building and dies at the buildings module's rate for that case. In the daytime runs of 12 November 2026 this count is lower than the count by cell by a tenth to a third, for a reason not yet found. The two agree at night")
            .view("counter", "in a building that collapsed", "agents.in_collapses", "people, counted by agent")
            .view("counter", "dead in collapses, by agent", "agents.dead_in_collapses", "")
            .view("counter", "displaced, by agent", "agents.displaced", "")
            .view("counter", "without power, by agent", "agents.without_power", "according to the electricity service at each agent's home cell")
            .view("counter", "on medical devices, without power", "agents.devices_without_power", "people who depend on electrically powered medical devices and have no grid electricity, counted by agent")
            .view("counter", "agent-days a second", "agents.per_second", "how fast the daily update of the agents runs")
            .view("map", "people without power, by agent", "agents_without_power", "per cell of the coarser electricity grid, each day")
            .view("chart", "without power, by agent", "agents.without_power", "")
            .view("chart", "displaced, by agent", "agents.displaced", "")
            .view("text", "what the agents do not yet do", "The agents take no actions of their own. None leaves the city, goes to a shelter, travels or waits at a hospital. Each one only records what happens at its home cell and its work cell. They are included so that a later version can give them a rule for deciding and a route on the road network, and so that the cost of updating about a hundred thousand agents for every simulated day is measured now.", "")
            .note("An agent that stands for many individuals is called a super-individual. The method comes from fish and plankton modelling in the 1990s (recalled from memory; no citation was checked). The agents are drawn using the run's seed, so the same seed gives the same agents.")
            .asset(&format!("events/{folder}"), "DOGAMI's maps of residents, daytime occupants and night-time occupants")
    }
    fn coarse(&self) -> LatLonGrid { let g = self.grid.as_ref().unwrap(); let step = (self.coarse_arcsec / self.res_arcsec).round().max(1.0) as usize; LatLonGrid { nx: (g.nx + step - 1) / step, ny: (g.ny + step - 1) / step, dlon: g.dlon * step as f64, dlat: g.dlat * step as f64, lon_w: g.lon_w, lat_n: g.lat_n } }
    fn count(&self, ctx: &mut Ctx) {
        let n = self.agents.len() as f64;
        let s = self.scale;
        ctx.report("agents.count", "agents", n); ctx.report("agents.people", "people", n * s);
        ctx.report("agents.dead", "people", self.agents.iter().filter(|a| a.dead).count() as f64 * s);
        ctx.report("agents.dead_in_collapses", "people", self.agents.iter().filter(|a| a.dead && a.in_collapse).count() as f64 * s);
        ctx.report("agents.in_collapses", "people", self.agents.iter().filter(|a| a.in_collapse).count() as f64 * s);
        ctx.report("agents.displaced", "people", self.agents.iter().filter(|a| a.displaced && !a.dead).count() as f64 * s);
        ctx.report("agents.without_power", "people", self.agents.iter().filter(|a| !a.power && !a.dead).count() as f64 * s);
        ctx.report("agents.without_water", "people", self.agents.iter().filter(|a| !a.water && !a.dead).count() as f64 * s);
        ctx.report("agents.devices_without_power", "people", self.agents.iter().filter(|a| a.device && !a.power && !a.dead).count() as f64 * s);
        let cg = self.coarse(); let g = self.grid.as_ref().unwrap();
        let step = (self.coarse_arcsec / self.res_arcsec).round().max(1.0) as usize;
        let mut f = vec![0.0f32; cg.nx * cg.ny];
        for a in &self.agents { if !a.power && !a.dead { let (r, c) = (a.home as usize / g.nx, a.home as usize % g.nx); f[(r / step) * cg.nx + c / step] += s as f32; } }
        ctx.field("agents_without_power", "people", &cg, f);
    }
}

impl Module for Agents {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("quake.toll"); ctx.subscribe("lifeline.state");
        if self.agents.is_empty() {
            let g = self.grid.clone().unwrap();
            let z = |v: Vec<f32>| -> Vec<f64> { v.into_iter().map(|x| if x.is_nan() || x < 0.0 { 0.0 } else { x as f64 }).collect() };
            let res = z(sample(&self.folder, "people_resident.tif", &g).unwrap_or_default());
            let day = z(sample(&self.folder, "people_day.tif", &g).unwrap_or_default());
            // cumulative tables, so a draw lands on a cell in proportion to its people
            let cum = |v: &[f64]| -> Vec<f64> { let mut c = Vec::with_capacity(v.len()); let mut s = 0.0; for x in v { s += x; c.push(s); } c };
            let (cr, cd) = (cum(&res), cum(&day));
            let (tr, td) = (*cr.last().unwrap_or(&0.0), *cd.last().unwrap_or(&0.0));
            let pick = |c: &[f64], total: f64, u: f64| -> u32 { let t = u * total; match c.binary_search_by(|x| x.partial_cmp(&t).unwrap()) { Ok(i) => i as u32, Err(i) => i.min(c.len() - 1) as u32 } };
            let n = (tr / self.scale).round() as usize;
            let mut agents = Vec::with_capacity(n);
            for _ in 0..n {
                let (u1, u2, u3) = (ctx.rng().f64(), ctx.rng().f64(), ctx.rng().f64());
                agents.push(Agent { home: pick(&cr, tr, u1), work: if td > 0.0 { pick(&cd, td, u2) } else { pick(&cr, tr, u2) }, device: u3 < self.device_share, dead: false, in_collapse: false, displaced: false, power: true, water: true });
            }
            self.agents = agents;
        }
        self.count(ctx);
        ctx.report("agents.per_second", "agents/s", 0.0);
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "quake.toll" => {
                if self.fired { return; }
                self.fired = true;
                let t = std::time::Instant::now();
                let n = self.grid.as_ref().map(|g| g.nx * g.ny).unwrap_or(0);
                let take = |name: &str| -> Vec<f32> { ctx.world_read(name).map(|f| f.data.clone()).unwrap_or_else(|| vec![0.0; n]) };
                let (cf, rc, ro, pr) = (take("quake.cell_collapse_frac"), take("quake.cell_death_rate_collapse"), take("quake.cell_death_rate_other"), take("quake.cell_displaced_rate"));
                let at_work = self.time_of_day == "day";
                for a in &mut self.agents {
                    let here = if at_work { a.work } else { a.home } as usize;
                    let (u1, u2, u3) = (ctx.rng().f64(), ctx.rng().f64(), ctx.rng().f64());
                    let f = cf.get(here).copied().unwrap_or(0.0) as f64;
                    a.in_collapse = !f.is_nan() && u1 < f;
                    let d = if a.in_collapse { rc.get(here).copied().unwrap_or(0.0) } else { ro.get(here).copied().unwrap_or(0.0) } as f64;
                    let p = pr.get(a.home as usize).copied().unwrap_or(0.0) as f64;
                    if !d.is_nan() && u2 < d { a.dead = true; }
                    if !p.is_nan() && u3 < p { a.displaced = true; }
                }
                let dt = t.elapsed().as_secs_f64().max(1e-6);
                ctx.report("agents.per_second", "agents/s", self.agents.len() as f64 / dt);
                self.count(ctx);
                let s = self.scale;
                ctx.publish("agents.toll", json!({ "dead": self.agents.iter().filter(|a| a.dead).count() as f64 * s, "dead_in_collapses": self.agents.iter().filter(|a| a.dead && a.in_collapse).count() as f64 * s, "displaced": self.agents.iter().filter(|a| a.displaced && !a.dead).count() as f64 * s }));
                if !self.scheduled { self.scheduled = true; ctx.schedule_with_priority(Duration::days(1), 6, "day", json!(null)); }
            }
            "lifeline.state" => { self.water_share = ev.data.get("water_share").and_then(|v| v.as_f64()).unwrap_or(1.0); }
            "day" => {
                self.day += 1;
                let t = std::time::Instant::now();
                let cg = self.coarse(); let g = self.grid.clone().unwrap();
                let step = (self.coarse_arcsec / self.res_arcsec).round().max(1.0) as usize;
                let up: Option<Vec<f32>> = ctx.world_read("power.up").map(|f| f.data.clone());
                let ws = self.water_share;
                for a in &mut self.agents {
                    if a.dead { continue; }
                    let (r, c) = (a.home as usize / g.nx, a.home as usize % g.nx);
                    let ci = (r / step) * cg.nx + c / step;
                    a.power = up.as_ref().map(|u| u.get(ci).copied().unwrap_or(1.0) >= 0.5).unwrap_or(true);
                    // the water ramp is a share, not a map: the same share of agents, by a stable draw on the home cell
                    a.water = ((a.home as f64 * 0.618_033_988_7) % 1.0) < ws;
                }
                let dt = t.elapsed().as_secs_f64().max(1e-6);
                ctx.report("agents.per_second", "agents/s", self.agents.len() as f64 / dt);
                self.count(ctx);
                ctx.schedule_with_priority(Duration::days(1), 6, "day", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Agents = serde_json::from_value(v).unwrap(); o.grid = window_grid(o.lat_s, o.lat_n, o.lon_w, o.lon_e, o.res_arcsec).ok().map(|x| x.0); *self = o; }
}
