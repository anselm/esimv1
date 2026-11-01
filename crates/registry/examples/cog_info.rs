//! cargo run -p esim-registry --example cog_info -- <file.tif>: what our reader makes of a GeoTIFF.
fn main() -> Result<(), String> {
    let p = std::env::args().nth(1).ok_or("path")?;
    let r = esim_registry::cog::read(std::path::Path::new(&p))?;
    let (lo, hi, mean, n) = r.stats();
    println!("{}x{} origin ({:.3}, {:.3}) pixel ({:.6}, {:.6}) nodata {:?}", r.width, r.height, r.x0, r.y0, r.dx, r.dy, r.nodata);
    println!("min {lo} max {hi} mean {mean:.3} valid {n}");
    Ok(())
}
