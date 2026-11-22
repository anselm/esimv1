#!/usr/bin/env python3
"""Cut NOAA's VIIRS I-band active-fire detections to a window and a span of days, as one CSV.

Source: the NOAA JPSS open-data buckets (noaa-nesdis-snpp-pds, noaa-nesdis-n20-pds,
noaa-nesdis-n21-pds), product VIIRS_EFIRE_VIIRSI_EDR, read anonymously over HTTPS. Each file
is 85 seconds of one satellite's swath and lists its fire pixels (FP_* variables). A file's name
does not say where it looked, so every file whose start falls in the hours a satellite can see
the window is fetched, read and thrown away; only the rows inside the window are kept.

Needs h5py and numpy. Written 2026-11-19 by Claude for the Palisades scenario, on Anselm's order.

  python3 fetch.py <out.csv> <lat_s> <lat_n> <lon_w> <lon_e> <first day> <last day> [tmp dir]
"""
import sys, os, re, urllib.request, tempfile, datetime as dt
from concurrent.futures import ThreadPoolExecutor
import numpy as np, h5py

out, lat_s, lat_n, lon_w, lon_e = sys.argv[1], *map(float, sys.argv[2:6])
d0, d1 = (dt.date.fromisoformat(s) for s in sys.argv[6:8])
tmp = sys.argv[8] if len(sys.argv) > 8 else tempfile.mkdtemp()
BUCKETS = {"SNPP": "noaa-nesdis-snpp-pds", "NOAA-20": "noaa-nesdis-n20-pds", "NOAA-21": "noaa-nesdis-n21-pds"}
# a sun-synchronous afternoon orbit crosses a longitude near 13:30 and 01:30 local solar time; the
# swath is wide enough that the orbit before and the orbit after also see it, so two hours either side
mid = [(13.5 - (lon_w + lon_e) / 2 / 15) % 24, (1.5 - (lon_w + lon_e) / 2 / 15) % 24]
def wanted(hhmm):
    h = int(hhmm[:2]) + int(hhmm[2:]) / 60
    return any(min(abs(h - m), 24 - abs(h - m)) <= 2.0 for m in mid)

def keys(bucket, day):
    prefix, token, got = f"VIIRS_EFIRE_VIIRSI_EDR/{day:%Y/%m/%d}/", None, []
    while True:
        url = f"https://{bucket}.s3.amazonaws.com/?list-type=2&prefix={prefix}" + (f"&continuation-token={urllib.parse.quote(token)}" if token else "")
        xml = urllib.request.urlopen(url, timeout=60).read().decode()
        got += re.findall(r"<Key>([^<]+)</Key>", xml)
        m = re.search(r"<NextContinuationToken>([^<]+)<", xml)
        if not m: return got
        token = m.group(1)

def rows(job):
    sat, bucket, key = job
    path = os.path.join(tmp, os.path.basename(key))
    for attempt in range(4):
        try:
            urllib.request.urlretrieve(f"https://{bucket}.s3.amazonaws.com/{key}", path); break
        except Exception as e:
            if attempt == 3: return [("ERROR", key, str(e))]
    res = []
    try:
        with h5py.File(path) as f:
            if "FP_Latitude" in f and f["FP_Latitude"].shape[0]:
                la, lo = f["FP_Latitude"][:], f["FP_Longitude"][:]
                m = (la >= lat_s) & (la <= lat_n) & (lo >= lon_w) & (lo <= lon_e)
                if m.any():
                    # the file covers 85 seconds; its middle is taken as every pixel's time
                    a = dt.datetime.strptime(f.attrs["time_coverage_start"].decode(), "%Y-%m-%dT%H:%M:%SZ"); b = dt.datetime.strptime(f.attrs["time_coverage_end"].decode(), "%Y-%m-%dT%H:%M:%SZ")
                    t = (a + (b - a) / 2).strftime("%Y-%m-%d %H:%M:%S")
                    g = lambda n: f[n][:][m]
                    for x in zip(la[m], lo[m], g("FP_FireRadiativePower"), g("FP_Confidence"), g("FP_FireMask"), g("FP_PixelArea"), g("FP_SatZenithAngle"), g("FP_Day")):
                        res.append((t, sat, "VIIRS-I", f"{x[0]:.5f}", f"{x[1]:.5f}", f"{x[2]:.2f}", int(x[3]), int(x[4]), f"{x[5]:.4f}", f"{x[6]:.1f}", int(x[7]), os.path.basename(key)))
    except Exception as e:
        res = [("ERROR", key, str(e))]
    os.remove(path)
    return res

import urllib.parse
jobs = []
day = d0
while day <= d1:
    for sat, b in BUCKETS.items():
        ks = [k for k in keys(b, day) if (m := re.search(r"_s\d{8}(\d{4})", k)) and wanted(m.group(1))]
        jobs += [(sat, b, k) for k in ks]
    day += dt.timedelta(days=1)
print(len(jobs), "files to read", file=sys.stderr)
allrows, errs = [], []
with ThreadPoolExecutor(24) as ex:
    for r in ex.map(rows, jobs):
        for x in r: (errs if x[0] == "ERROR" else allrows).append(x)
allrows.sort()
with open(out, "w") as f:
    f.write("rep_date,satellite,sensor,lat,lon,frp_mw,confidence,fire_mask,pixel_area_km2,sat_zenith_deg,day,file\n")
    for r in allrows: f.write(",".join(map(str, r)) + "\n")
print(len(allrows), "detections,", len(errs), "files failed", file=sys.stderr)
for e in errs[:10]: print(e, file=sys.stderr)
