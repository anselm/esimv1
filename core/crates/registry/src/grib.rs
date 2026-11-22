//! A small GRIB2 reader for what NOAA's HRRR surface files are: one message per record,
//! grid definition template 3.30 (Lambert conformal on a sphere), data representation
//! template 5.0 (simple packing) or 5.2/5.3 (complex packing, with spatial differencing), no
//! bitmap. Enough to read a 10 m wind, a gust, a
//! temperature or a humidity record fetched by byte range from the open bucket, and to
//! say where each grid point is. Not a general GRIB library; it says so when handed
//! anything else.

use serde::{Deserialize, Serialize};

/// Lambert conformal conic on a sphere, as HRRR declares it: the first grid point
/// (south-west), the central meridian, the standard parallels, the spacing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lcc {
    pub nx: usize, pub ny: usize,
    pub la1: f64, pub lo1: f64, pub lov: f64, pub latin1: f64, pub latin2: f64,
    pub dx_m: f64, pub dy_m: f64,
    pub radius_m: f64,
}

impl Lcc {
    fn n(&self) -> f64 {
        let (p1, p2) = (self.latin1.to_radians(), self.latin2.to_radians());
        if (p1 - p2).abs() < 1e-9 { p1.sin() } else { (p1.cos() / p2.cos()).ln() / ((std::f64::consts::FRAC_PI_4 + p2 / 2.0).tan() / (std::f64::consts::FRAC_PI_4 + p1 / 2.0).tan()).ln() }
    }
    fn xy(&self, lat: f64, lon: f64) -> (f64, f64) {
        let n = self.n();
        let p1 = self.latin1.to_radians();
        let f = p1.cos() * (std::f64::consts::FRAC_PI_4 + p1 / 2.0).tan().powf(n) / n;
        let rho = self.radius_m * f / (std::f64::consts::FRAC_PI_4 + lat.to_radians() / 2.0).tan().powf(n);
        let mut lov = self.lov; if lov > 180.0 { lov -= 360.0; }
        let th = n * (lon - lov).to_radians();
        (rho * th.sin(), -rho * th.cos())
    }
    /// Fractional grid indices (i along x, j along y from the south-west corner) of a point.
    pub fn ij(&self, lat: f64, lon: f64) -> (f64, f64) {
        let mut lo1 = self.lo1; if lo1 > 180.0 { lo1 -= 360.0; }
        let (x0, y0) = self.xy(self.la1, lo1);
        let (x, y) = self.xy(lat, lon);
        ((x - x0) / self.dx_m, (y - y0) / self.dy_m)
    }
}

/// One decoded record: the grid and every value, row-major from the south-west corner.
pub struct Record { pub grid: Lcc, pub values: Vec<f32> }

fn be32(b: &[u8]) -> u32 { u32::from_be_bytes([b[0], b[1], b[2], b[3]]) }
fn be16(b: &[u8]) -> u16 { u16::from_be_bytes([b[0], b[1]]) }
/// GRIB's signed integers are sign-and-magnitude.
fn sm16(b: &[u8]) -> i32 { let v = be16(b); if v & 0x8000 != 0 { -((v & 0x7fff) as i32) } else { v as i32 } }
fn sm32(b: &[u8]) -> i64 { let v = be32(b); if v & 0x8000_0000 != 0 { -((v & 0x7fff_ffff) as i64) } else { v as i64 } }

pub fn read(buf: &[u8]) -> Result<Record, String> {
    if buf.len() < 16 || &buf[..4] != b"GRIB" { return Err("not a GRIB message".into()); }
    if buf[7] != 2 { return Err(format!("GRIB edition {}", buf[7])); }
    let mut i = 16;
    let (mut grid, mut npts, mut pack, mut data, mut bitmap): (Option<Lcc>, usize, Option<Vec<u8>>, Option<&[u8]>, bool) = (None, 0usize, None, None, false);
    while i + 5 <= buf.len() {
        if &buf[i..i.min(buf.len()).min(i + 4)] == b"7777" { break; }
        let len = be32(&buf[i..]) as usize;
        let sec = buf[i + 4];
        let s = buf.get(i..i + len).ok_or("section out of range")?;
        match sec {
            3 => {
                let tmpl = be16(&s[12..]);
                if tmpl != 30 { return Err(format!("grid template {tmpl}; only Lambert conformal (30) here")); }
                let shape = s[14];
                let radius_m = match shape { 6 => 6_371_229.0, 0 => 6_367_470.0, 1 => { let sf = s[15] as i32; sm32(&s[16..]) as f64 / 10f64.powi(sf) } _ => return Err(format!("earth shape {shape}")) };
                let o = 30;
                let g = Lcc {
                    nx: be32(&s[o..]) as usize, ny: be32(&s[o + 4..]) as usize,
                    la1: sm32(&s[o + 8..]) as f64 / 1e6, lo1: sm32(&s[o + 12..]) as f64 / 1e6,
                    lov: sm32(&s[o + 21..]) as f64 / 1e6,
                    dx_m: be32(&s[o + 25..]) as f64 / 1e3, dy_m: be32(&s[o + 29..]) as f64 / 1e3,
                    latin1: sm32(&s[o + 35..]) as f64 / 1e6, latin2: sm32(&s[o + 39..]) as f64 / 1e6,
                    radius_m,
                };
                let scan = s[o + 34];
                if scan & 0x80 != 0 || scan & 0x40 == 0 || scan & 0x20 != 0 { return Err(format!("scanning mode {scan:#x}; only +i +j row-major here")); }
                grid = Some(g);
            }
            5 => {
                npts = be32(&s[5..]) as usize;
                let tmpl = be16(&s[9..]);
                if tmpl != 0 && tmpl != 2 && tmpl != 3 { return Err(format!("data representation template {tmpl}; only simple (0) and complex (2, 3) packing here")); }
                pack = Some(s.to_vec());
            }
            6 => { bitmap = s[5] != 255; }
            7 => { data = Some(&s[5..]); }
            _ => {}
        }
        i += len;
    }
    if bitmap { return Err("bitmapped record; not handled".into()); }
    let grid = grid.ok_or("no grid section")?;
    let sec5 = pack.ok_or("no packing section")?;
    let data = data.ok_or("no data section")?;
    if grid.nx * grid.ny != npts { return Err(format!("grid {}x{} but {npts} points", grid.nx, grid.ny)); }
    let tmpl = be16(&sec5[9..]);
    let r = f32::from_be_bytes([sec5[11], sec5[12], sec5[13], sec5[14]]) as f64;
    let e = sm16(&sec5[15..]); let d = sm16(&sec5[17..]); let bits = sec5[19] as u32;
    let scale = 2f64.powi(e) / 10f64.powi(d);
    let base = r / 10f64.powi(d);
    let values = if tmpl == 0 {
        if bits == 0 { vec![base as f32; npts] } else {
            let mut br = BitReader::new(data);
            (0..npts).map(|_| br.take(bits).map(|v| (base + v as f64 * scale) as f32)).collect::<Result<Vec<_>, _>>()?
        }
    } else {
        complex_unpack(&sec5, data, npts, tmpl == 3)?.into_iter().map(|y| (base + y as f64 * scale) as f32).collect()
    };
    Ok(Record { grid, values })
}

struct BitReader<'a> { data: &'a [u8], pos: usize }
impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self { BitReader { data, pos: 0 } }
    fn take(&mut self, bits: u32) -> Result<u64, String> {
        let mut v = 0u64;
        for _ in 0..bits {
            let byte = *self.data.get(self.pos >> 3).ok_or("data section short")?;
            v = (v << 1) | ((byte >> (7 - (self.pos & 7))) & 1) as u64;
            self.pos += 1;
        }
        Ok(v)
    }
    fn align(&mut self) { self.pos = (self.pos + 7) & !7; }
}

/// Templates 5.2 and 5.3: values split into groups, each with its own reference and
/// width, group lengths scaled by an increment; with 5.3 the stored values are first or
/// second differences from a stated start. Missing-value management is not handled.
fn complex_unpack(sec5: &[u8], data: &[u8], npts: usize, differenced: bool) -> Result<Vec<i64>, String> {
    let nbits_ref = sec5[19] as u32;
    let missing = sec5[22];
    if missing != 0 { return Err(format!("missing value management {missing}; not handled")); }
    let ng = be32(&sec5[31..]) as usize;
    let ref_gw = sec5[35] as u64; let nbits_gw = sec5[36] as u32;
    let ref_gl = be32(&sec5[37..]) as u64; let len_inc = sec5[41] as u64; let last_len = be32(&sec5[42..]) as u64; let nbits_gl = sec5[46] as u32;
    let (order, nbytes_extra) = if differenced { (sec5[47] as usize, sec5[48] as usize) } else { (0, 0) };
    let mut br = BitReader::new(data);
    // spatial differencing descriptors: the first `order` values and the minimum, each nbytes wide, sign-magnitude
    let mut firsts = Vec::new(); let mut gmin = 0i64;
    if differenced {
        let sm = |v: u64, nbytes: usize| -> i64 { let sign = 1u64 << (nbytes * 8 - 1); if v & sign != 0 { -((v & (sign - 1)) as i64) } else { v as i64 } };
        for _ in 0..order { firsts.push(sm(br.take(nbytes_extra as u32 * 8)?, nbytes_extra)); }
        gmin = sm(br.take(nbytes_extra as u32 * 8)?, nbytes_extra);
    }
    let mut refs = Vec::with_capacity(ng); for _ in 0..ng { refs.push(br.take(nbits_ref)?); } br.align();
    let mut widths = Vec::with_capacity(ng); for _ in 0..ng { widths.push(ref_gw + br.take(nbits_gw)?); } br.align();
    let mut lens = Vec::with_capacity(ng); for _ in 0..ng { lens.push(ref_gl + br.take(nbits_gl)? * len_inc); } br.align();
    if let Some(l) = lens.last_mut() { *l = last_len; }
    let mut out: Vec<i64> = Vec::with_capacity(npts);
    for g in 0..ng {
        for _ in 0..lens[g] {
            let v = if widths[g] == 0 { 0 } else { br.take(widths[g] as u32)? };
            out.push(refs[g] as i64 + v as i64);
        }
    }
    if out.len() != npts { return Err(format!("complex packing: {} values for {npts} points", out.len())); }
    if differenced {
        // the stored values are differences plus gmin; the first `order` values are stated outright
        for v in out.iter_mut() { *v += gmin; }
        match order {
            1 => { out[0] = firsts[0]; for i in 1..npts { out[i] += out[i - 1]; } }
            2 => { out[0] = firsts[0]; out[1] = firsts[1]; for i in 2..npts { out[i] += 2 * out[i - 1] - out[i - 2]; } }
            o => return Err(format!("spatial differencing order {o}")),
        }
    }
    Ok(out)
}

/// A record's index line, as the `.idx` beside each HRRR file lists them:
/// `n:offset:d=YYYYMMDDHH:VAR:level:forecast:`.
pub struct IdxEntry { pub n: usize, pub offset: u64, pub var: String, pub level: String, pub forecast: String }

pub fn parse_idx(text: &str) -> Vec<IdxEntry> {
    text.lines().filter_map(|l| {
        let p: Vec<&str> = l.split(':').collect();
        if p.len() < 6 { return None; }
        Some(IdxEntry { n: p[0].parse().ok()?, offset: p[1].parse().ok()?, var: p[3].to_string(), level: p[4].to_string(), forecast: p[5].to_string() })
    }).collect()
}

/// Byte range of record `n` (its offset to the next record's offset minus one, or to the
/// end of the file when it is the last).
pub fn range_of(entries: &[IdxEntry], n: usize, file_len: Option<u64>) -> Option<(u64, u64)> {
    let i = entries.iter().position(|e| e.n == n)?;
    let start = entries[i].offset;
    let end = entries.get(i + 1).map(|e| e.offset - 1).or(file_len.map(|l| l - 1)).unwrap_or(u64::MAX / 2);
    Some((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn idx_lines_parse() {
        let e = parse_idx("77:44618053:d=2025010718:UGRD:10 m above ground:anl:\n78:46999668:d=2025010718:VGRD:10 m above ground:anl:\n");
        assert_eq!(e.len(), 2); assert_eq!(e[0].var, "UGRD"); assert_eq!(range_of(&e, 77, None), Some((44618053, 46999667)));
    }
    #[test]
    fn lambert_round_trip_is_near_the_corner() {
        let g = Lcc { nx: 1799, ny: 1059, la1: 21.138123, lo1: 237.280472, lov: 262.5, latin1: 38.5, latin2: 38.5, dx_m: 3000.0, dy_m: 3000.0, radius_m: 6_371_229.0 };
        let (i, j) = g.ij(21.138123, 237.280472 - 360.0);
        assert!(i.abs() < 1e-6 && j.abs() < 1e-6, "{i} {j}");
    }
}
