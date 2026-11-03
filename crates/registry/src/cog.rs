//! A small reader for the GeoTIFFs the registry actually uses: single-band, tiled,
//! little- or big-endian, uncompressed, deflate or LZW, with none, horizontal or
//! floating-point predictor, Float32 or Int16 samples. Not a general TIFF library; it
//! reads what Copernicus DEM tiles are, and says so when handed anything else.

use flate2::read::ZlibDecoder;
use std::io::Read;

#[derive(Clone, Debug)]
pub struct Raster {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
    pub nodata: Option<f32>,
    /// Longitude of the west edge, latitude of the north edge, pixel width, pixel height (positive).
    pub x0: f64,
    pub y0: f64,
    pub dx: f64,
    pub dy: f64,
}

impl Raster {
    pub fn get(&self, col: usize, row: usize) -> f32 { self.data[row * self.width + col] }
    /// Pixel centre coordinates.
    pub fn lon(&self, col: usize) -> f64 { self.x0 + (col as f64 + 0.5) * self.dx }
    pub fn lat(&self, row: usize) -> f64 { self.y0 - (row as f64 + 0.5) * self.dy }
    pub fn stats(&self) -> (f32, f32, f64, usize) {
        let (mut lo, mut hi, mut sum, mut n) = (f32::MAX, f32::MIN, 0.0f64, 0usize);
        for &v in &self.data {
            if Some(v) == self.nodata || !v.is_finite() { continue; }
            lo = lo.min(v); hi = hi.max(v); sum += v as f64; n += 1;
        }
        (lo, hi, if n > 0 { sum / n as f64 } else { f64::NAN }, n)
    }
}

struct Ifd { tags: std::collections::BTreeMap<u16, (u16, u32, Vec<u8>)> }

fn rd16(b: &[u8], le: bool) -> u16 { if le { u16::from_le_bytes([b[0], b[1]]) } else { u16::from_be_bytes([b[0], b[1]]) } }
fn rd32(b: &[u8], le: bool) -> u32 { if le { u32::from_le_bytes([b[0], b[1], b[2], b[3]]) } else { u32::from_be_bytes([b[0], b[1], b[2], b[3]]) } }
fn rd64f(b: &[u8], le: bool) -> f64 { let a: [u8; 8] = b[..8].try_into().unwrap(); if le { f64::from_le_bytes(a) } else { f64::from_be_bytes(a) } }

fn type_size(t: u16) -> usize { match t { 1 | 2 | 6 | 7 => 1, 3 | 8 => 2, 4 | 9 | 11 => 4, 5 | 10 | 12 | 16 => 8, _ => 1 } }

impl Ifd {
    fn parse(buf: &[u8], le: bool, off: usize) -> Result<Ifd, String> {
        let n = rd16(&buf[off..], le) as usize;
        let mut tags = std::collections::BTreeMap::new();
        for i in 0..n {
            let e = &buf[off + 2 + i * 12..off + 14 + i * 12];
            let tag = rd16(e, le); let typ = rd16(&e[2..], le); let cnt = rd32(&e[4..], le);
            let sz = type_size(typ) * cnt as usize;
            let val = if sz <= 4 { e[8..8 + sz].to_vec() } else {
                let vo = rd32(&e[8..], le) as usize;
                buf.get(vo..vo + sz).ok_or("ifd value out of range")?.to_vec()
            };
            tags.insert(tag, (typ, cnt, val));
        }
        Ok(Ifd { tags })
    }
    fn u32s(&self, tag: u16, le: bool) -> Vec<u32> {
        match self.tags.get(&tag) {
            Some((3, cnt, v)) => (0..*cnt as usize).map(|i| rd16(&v[i * 2..], le) as u32).collect(),
            Some((4, cnt, v)) => (0..*cnt as usize).map(|i| rd32(&v[i * 4..], le)).collect(),
            _ => vec![],
        }
    }
    fn u32(&self, tag: u16, le: bool, default: u32) -> u32 { self.u32s(tag, le).first().copied().unwrap_or(default) }
    fn f64s(&self, tag: u16, le: bool) -> Vec<f64> {
        match self.tags.get(&tag) { Some((12, cnt, v)) => (0..*cnt as usize).map(|i| rd64f(&v[i * 8..], le)).collect(), _ => vec![] }
    }
    fn ascii(&self, tag: u16) -> Option<String> {
        self.tags.get(&tag).map(|(_, _, v)| String::from_utf8_lossy(v).trim_end_matches('\0').to_string())
    }
}

/// Read the first image of a GeoTIFF file into a Raster.
pub fn read(path: &std::path::Path) -> Result<Raster, String> {
    let buf = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    read_bytes(&buf)
}

pub fn read_bytes(buf: &[u8]) -> Result<Raster, String> {
    if buf.len() < 8 { return Err("not a tiff".into()); }
    let le = match &buf[..2] { b"II" => true, b"MM" => false, _ => return Err("not a tiff".into()) };
    if rd16(&buf[2..], le) != 42 { return Err("bigtiff not supported".into()); }
    let ifd = Ifd::parse(buf, le, rd32(&buf[4..], le) as usize)?;
    let width = ifd.u32(256, le, 0) as usize;
    let height = ifd.u32(257, le, 0) as usize;
    let bits = ifd.u32(258, le, 0);
    let comp = ifd.u32(259, le, 1);
    let spp = ifd.u32(277, le, 1);
    let pred = ifd.u32(317, le, 1);
    let fmt = ifd.u32(339, le, 1);
    if spp != 1 { return Err(format!("samples per pixel {spp}; only single band supported")); }
    let bps = (bits / 8) as usize;
    let to_f32: fn(&[u8], bool) -> f32 = match (fmt, bits) {
        (3, 32) => |b, le| { let a = [b[0], b[1], b[2], b[3]]; if le { f32::from_le_bytes(a) } else { f32::from_be_bytes(a) } },
        (2, 16) => |b, le| (if le { i16::from_le_bytes([b[0], b[1]]) } else { i16::from_be_bytes([b[0], b[1]]) }) as f32,
        (1, 16) => |b, le| rd16(b, le) as f32,
        (1, 8) => |b, _| b[0] as f32,
        _ => return Err(format!("sample format {fmt} with {bits} bits not supported")),
    };
    let nodata = ifd.ascii(42113).and_then(|s| s.trim().parse::<f32>().ok());
    let scale = ifd.f64s(33550, le);
    let tie = ifd.f64s(33922, le);
    let (dx, dy) = (scale.first().copied().unwrap_or(1.0), scale.get(1).copied().unwrap_or(1.0));
    let (mut x0, mut y0) = if tie.len() >= 6 { (tie[3] - tie[0] * dx, tie[4] + tie[1] * dy) } else { (0.0, 0.0) };
    // GTRasterTypeGeoKey (1025): 1 = PixelIsArea (tiepoint at the corner), 2 = PixelIsPoint
    // (tiepoint at the centre of the first pixel). Copernicus tiles are PixelIsPoint, so
    // the outer edge is half a pixel beyond the tiepoint.
    if raster_type(&ifd, le) == 2 { x0 -= dx / 2.0; y0 += dy / 2.0; }

    let mut data = vec![f32::NAN; width * height];
    let (tw, th) = (ifd.u32(322, le, 0) as usize, ifd.u32(323, le, 0) as usize);
    if tw > 0 {
        let offs = ifd.u32s(324, le);
        let counts = ifd.u32s(325, le);
        let tiles_across = width.div_ceil(tw);
        for (i, (&o, &c)) in offs.iter().zip(counts.iter()).enumerate() {
            let raw = buf.get(o as usize..(o + c) as usize).ok_or("tile out of range")?;
            let mut block = decompress(raw, comp, tw * th * bps)?;
            unpredict(&mut block, pred, tw, th, bps, le)?;
            let (tx, ty) = (i % tiles_across, i / tiles_across);
            for r in 0..th {
                let row = ty * th + r;
                if row >= height { break; }
                for cidx in 0..tw {
                    let col = tx * tw + cidx;
                    if col >= width { break; }
                    let p = (r * tw + cidx) * bps;
                    data[row * width + col] = to_f32(&block[p..p + bps], le);
                }
            }
        }
    } else {
        let offs = ifd.u32s(273, le);
        let counts = ifd.u32s(279, le);
        let rps = ifd.u32(278, le, height as u32) as usize;
        for (i, (&o, &c)) in offs.iter().zip(counts.iter()).enumerate() {
            let raw = buf.get(o as usize..(o + c) as usize).ok_or("strip out of range")?;
            let rows = rps.min(height - i * rps);
            let mut block = decompress(raw, comp, width * rows * bps)?;
            unpredict(&mut block, pred, width, rows, bps, le)?;
            for r in 0..rows {
                for col in 0..width {
                    let p = (r * width + col) * bps;
                    data[(i * rps + r) * width + col] = to_f32(&block[p..p + bps], le);
                }
            }
        }
    }
    Ok(Raster { width, height, data, nodata, x0, y0, dx, dy })
}

/// Value of GeoKey 1025 (raster type) from the GeoKeyDirectory, or 1 if absent.
fn raster_type(ifd: &Ifd, le: bool) -> u32 {
    let keys = ifd.u32s(34735, le);
    if keys.len() < 4 { return 1; }
    let n = keys[3] as usize;
    for k in 0..n {
        let e = &keys[4 + k * 4..];
        if e.len() < 4 { break; }
        if e[0] == 1025 && e[1] == 0 { return e[3]; }
    }
    1
}

fn decompress(raw: &[u8], comp: u32, expect: usize) -> Result<Vec<u8>, String> {
    match comp {
        1 => Ok(raw.to_vec()),
        8 | 32946 => {
            let mut out = Vec::with_capacity(expect);
            ZlibDecoder::new(raw).read_to_end(&mut out).map_err(|e| format!("deflate: {e}"))?;
            Ok(out)
        }
        5 => lzw_decode(raw, expect),
        other => Err(format!("compression {other} not supported (only none, deflate and LZW)")),
    }
}

/// TIFF LZW (compression 5): MSB-first codes from 9 to 12 bits, clear 256, end 257, the
/// width growing one code early as TIFF writers do. Enough for WorldPop's tiles.
pub fn lzw_decode(raw: &[u8], expect: usize) -> Result<Vec<u8>, String> {
    const CLEAR: usize = 256; const EOI: usize = 257;
    let mut out: Vec<u8> = Vec::with_capacity(expect);
    // table entries: (prefix code, last byte, length); 0..256 are the literals
    let mut prefix: Vec<u32> = Vec::with_capacity(4096);
    let mut last: Vec<u8> = Vec::with_capacity(4096);
    let mut len: Vec<u32> = Vec::with_capacity(4096);
    let reset = |prefix: &mut Vec<u32>, last: &mut Vec<u8>, len: &mut Vec<u32>| {
        prefix.clear(); last.clear(); len.clear();
        for i in 0..258u32 { prefix.push(u32::MAX); last.push(i as u8); len.push(1); }
    };
    reset(&mut prefix, &mut last, &mut len);
    let (mut bitpos, mut width) = (0usize, 9usize);
    let nbits = raw.len() * 8;
    let next = |bitpos: &mut usize, width: usize| -> Option<usize> {
        if *bitpos + width > nbits { return None; }
        let mut v = 0usize;
        for _ in 0..width { let b = (raw[*bitpos / 8] >> (7 - *bitpos % 8)) & 1; v = (v << 1) | b as usize; *bitpos += 1; }
        Some(v)
    };
    // write the string for `code` at the end of `out`, returning its first byte
    let emit = |code: usize, out: &mut Vec<u8>, prefix: &[u32], last: &[u8], len: &[u32]| -> u8 {
        let n = len[code] as usize;
        let start = out.len();
        out.resize(start + n, 0);
        let mut c = code;
        for i in (0..n).rev() { out[start + i] = last[c]; c = prefix[c] as usize; }
        out[start]
    };
    let mut old: Option<usize> = None;
    while let Some(code) = next(&mut bitpos, width) {
        if code == EOI { break; }
        if code == CLEAR { reset(&mut prefix, &mut last, &mut len); width = 9; old = None; continue; }
        match old {
            None => { if code >= prefix.len() { return Err("lzw: bad first code".into()); } emit(code, &mut out, &prefix, &last, &len); old = Some(code); }
            Some(o) => {
                let first = if code < prefix.len() {
                    emit(code, &mut out, &prefix, &last, &len)
                } else if code == prefix.len() {
                    // the KwKwK case: old string plus its own first byte
                    let f = emit(o, &mut out, &prefix, &last, &len);
                    out.push(f); f
                } else { return Err("lzw: code beyond table".into()) };
                prefix.push(o as u32); last.push(first); len.push(len[o] + 1);
                old = Some(code);
                let n = prefix.len();
                if n + 1 >= (1 << width) && width < 12 { width += 1; }
            }
        }
        if out.len() >= expect { break; }
    }
    Ok(out)
}

/// Walk a tiled single-band GeoTIFF tile by tile without holding the whole raster:
/// `visit(col0, row0, tile_w, tile_h, samples)` gets each decoded tile as f32 in row-major
/// order (tile_w wide, tile_h high, both clipped to the image). For rasters too large to
/// read whole, such as a 1 km world grid.
pub struct TiledInfo { pub width: usize, pub height: usize, pub nodata: Option<f32>, pub x0: f64, pub y0: f64, pub dx: f64, pub dy: f64, pub tile_w: usize, pub tile_h: usize }
pub fn for_each_tile(buf: &[u8], mut visit: impl FnMut(&TiledInfo, usize, usize, usize, usize, &[f32])) -> Result<TiledInfo, String> {
    if buf.len() < 8 { return Err("not a tiff".into()); }
    let le = match &buf[..2] { b"II" => true, b"MM" => false, _ => return Err("not a tiff".into()) };
    if rd16(&buf[2..], le) != 42 { return Err("bigtiff not supported".into()); }
    let ifd = Ifd::parse(buf, le, rd32(&buf[4..], le) as usize)?;
    let width = ifd.u32(256, le, 0) as usize;
    let height = ifd.u32(257, le, 0) as usize;
    let bits = ifd.u32(258, le, 0);
    let comp = ifd.u32(259, le, 1);
    let pred = ifd.u32(317, le, 1);
    let fmt = ifd.u32(339, le, 1);
    if ifd.u32(277, le, 1) != 1 { return Err("only single band supported".into()); }
    if (fmt, bits) != (3, 32) { return Err(format!("sample format {fmt} with {bits} bits: only Float32 here")); }
    let nodata = ifd.ascii(42113).and_then(|s| s.trim().parse::<f32>().ok());
    let scale = ifd.f64s(33550, le);
    let tie = ifd.f64s(33922, le);
    let (dx, dy) = (scale.first().copied().unwrap_or(1.0), scale.get(1).copied().unwrap_or(1.0));
    let (mut x0, mut y0) = if tie.len() >= 6 { (tie[3] - tie[0] * dx, tie[4] + tie[1] * dy) } else { (0.0, 0.0) };
    if raster_type(&ifd, le) == 2 { x0 -= dx / 2.0; y0 += dy / 2.0; }
    let (tw, th) = (ifd.u32(322, le, 0) as usize, ifd.u32(323, le, 0) as usize);
    if tw == 0 { return Err("not tiled".into()); }
    let info = TiledInfo { width, height, nodata, x0, y0, dx, dy, tile_w: tw, tile_h: th };
    let offs = ifd.u32s(324, le);
    let counts = ifd.u32s(325, le);
    let tiles_across = width.div_ceil(tw);
    let mut samples = vec![0f32; tw * th];
    for (i, (&o, &c)) in offs.iter().zip(counts.iter()).enumerate() {
        let (tx, ty) = (i % tiles_across, i / tiles_across);
        let (col0, row0) = (tx * tw, ty * th);
        if c == 0 { continue; } // an empty tile: nothing there
        let raw = buf.get(o as usize..(o + c) as usize).ok_or("tile out of range")?;
        let mut block = decompress(raw, comp, tw * th * 4)?;
        if block.len() < tw * th * 4 { block.resize(tw * th * 4, 0); }
        unpredict(&mut block, pred, tw, th, 4, le)?;
        for k in 0..tw * th { let b = &block[k * 4..k * 4 + 4]; let a = [b[0], b[1], b[2], b[3]]; samples[k] = if le { f32::from_le_bytes(a) } else { f32::from_be_bytes(a) }; }
        let w = tw.min(width - col0); let h = th.min(height - row0);
        visit(&info, col0, row0, w, h, &samples);
    }
    Ok(info)
}

/// Undo TIFF predictors in place. 2 = horizontal differencing on samples, 3 = floating
/// point: bytes of each row are stored as byte planes (most significant first) with
/// horizontal differencing over the byte stream; undo the differencing, then gather.
fn unpredict(block: &mut [u8], pred: u32, w: usize, h: usize, bps: usize, le: bool) -> Result<(), String> {
    match pred {
        1 => Ok(()),
        2 => {
            if bps != 2 { return Err("horizontal predictor only implemented for 16-bit".into()); }
            for r in 0..h {
                let row = &mut block[r * w * 2..(r + 1) * w * 2];
                let mut prev: u16 = 0;
                for c in 0..w {
                    let v = rd16(&row[c * 2..], le).wrapping_add(prev);
                    row[c * 2..c * 2 + 2].copy_from_slice(&(if le { v.to_le_bytes() } else { v.to_be_bytes() }));
                    prev = v;
                }
            }
            Ok(())
        }
        3 => {
            let rowlen = w * bps;
            let mut tmp = vec![0u8; rowlen];
            for r in 0..h {
                let row = &mut block[r * rowlen..(r + 1) * rowlen];
                for i in 1..rowlen { row[i] = row[i].wrapping_add(row[i - 1]); }
                // row now holds byte planes: plane k holds byte k (big-endian order) of every sample
                for c in 0..w {
                    for k in 0..bps {
                        let be_byte = row[k * w + c];
                        // write sample bytes in the file's byte order
                        let pos = if le { c * bps + (bps - 1 - k) } else { c * bps + k };
                        tmp[pos] = be_byte;
                    }
                }
                row.copy_from_slice(&tmp);
            }
            Ok(())
        }
        other => Err(format!("predictor {other} not supported")),
    }
}
