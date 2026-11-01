//! cargo run -p esim-registry --example dem_stats -- <lat> <lon>
//! Fetches the Copernicus tile containing the point and prints its statistics.
use esim_registry::{catalog::tile_key, cog, Catalog, Fetcher};
use std::path::Path;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let lat: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(42.4319722);
    let lon: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(-93.8059444);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cat = Catalog::load(&root.join("data/catalog.toml"))?;
    let dem = cat.asset("copernicus-dem-glo30").ok_or("no dem asset")?;
    let key = tile_key(lat, lon);
    let url = dem.tile_url(&key)?;
    let mut f = Fetcher::default_for(&root)?;
    let t0 = std::time::Instant::now();
    let path = f.fetch(&format!("{}/{}.tif", dem.id, key), &url)?;
    println!("tile     {key}  {}  ({} ms)", path.display(), t0.elapsed().as_millis());
    let t1 = std::time::Instant::now();
    let r = cog::read(&path)?;
    let (lo, hi, mean, n) = r.stats();
    println!("raster   {}x{}  origin ({:.4}, {:.4})  pixel ({:.6}, {:.6})  nodata {:?}  decoded in {} ms",
        r.width, r.height, r.x0, r.y0, r.dx, r.dy, r.nodata, t1.elapsed().as_millis());
    println!("elev     min {lo:.1} m  max {hi:.1} m  mean {mean:.1} m  over {n} valid cells");
    // elevation at the point
    let col = ((lon - r.x0) / r.dx) as usize;
    let row = ((r.y0 - lat) / r.dy) as usize;
    println!("at point ({lat:.5}, {lon:.5}) -> col {col} row {row}: {:.1} m", r.get(col, row));
    Ok(())
}
