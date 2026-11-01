//! cargo run --release -p esim-registry --example basin -- lat_s lat_n lon_w lon_e res_arcsec gauge_lat gauge_lon
//! Delineates the basin above a gauge and prints its area against the USGS figure.
use esim_registry::{dem_window, Catalog, Fetcher, FlowNet};
use std::path::Path;

fn main() -> Result<(), String> {
    let a: Vec<f64> = std::env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
    let (lat_s, lat_n, lon_w, lon_e, res) = (a.first().copied().unwrap_or(42.3), a.get(1).copied().unwrap_or(43.2), a.get(2).copied().unwrap_or(-94.2), a.get(3).copied().unwrap_or(-93.5), a.get(4).copied().unwrap_or(15.0));
    let (glat, glon) = (a.get(5).copied().unwrap_or(42.4319722), a.get(6).copied().unwrap_or(-93.8059444));
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cat = Catalog::load(&root.join("data/catalog.toml"))?;
    let dem = cat.asset("copernicus-dem-glo30").ok_or("no dem")?;
    let mut f = Fetcher::default_for(&root)?;
    let t0 = std::time::Instant::now();
    let w = dem_window(&mut f, dem, lat_s, lat_n, lon_w, lon_e, res)?;
    println!("window   {}x{} cells at {res}\" ({} ms)", w.nx, w.ny, t0.elapsed().as_millis());
    let per_deg = esim_registry::EARTH_RADIUS_M * std::f64::consts::PI / 180.0;
    let row_area: Vec<f64> = (0..w.ny).map(|r| {
        let top = (w.lat_n - r as f64 * w.dlat).to_radians(); let bot = (w.lat_n - (r + 1) as f64 * w.dlat).to_radians();
        esim_registry::EARTH_RADIUS_M * esim_registry::EARTH_RADIUS_M * w.dlon.to_radians() * (top.sin() - bot.sin())
    }).collect();
    let _ = per_deg;
    let t1 = std::time::Instant::now();
    let net = FlowNet::build(&f, &w, &row_area);
    println!("flownet  built in {} ms", t1.elapsed().as_millis());
    let col = ((glon - w.lon_w) / w.dlon) as usize;
    let row = ((w.lat_n - glat) / w.dlat) as usize;
    for radius in [0usize, 2, 4, 6, 8] {
        let pour = net.snap(col, row, radius);
        let (pc, pr) = (pour % net.nx, pour / net.nx);
        let basin = net.basin(pour);
        let area_km2: f64 = basin.iter().enumerate().filter(|(_, &b)| b).map(|(i, _)| row_area[i / net.nx]).sum::<f64>() / 1e6;
        let touches_edge = basin.iter().enumerate().any(|(i, &b)| b && (i % net.nx == 0 || i / net.nx == 0 || i % net.nx == net.nx - 1 || i / net.nx == net.ny - 1));
        println!("snap r={radius}: pour at ({:.4}, {:.4}) accum {} cells, basin {:.0} km2 (USGS 844 sq mi = 2186 km2){}",
            w.lat(pr), w.lon(pc), net.accum[pour], area_km2, if touches_edge { "  TOUCHES WINDOW EDGE" } else { "" });
    }
    Ok(())
}
