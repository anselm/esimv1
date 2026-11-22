#!/usr/bin/env python3
"""Object storage for the hub: put project folders in a bucket as the site serves them.

  store.py buckets                        what the credential can see
  store.py put <area/name> [run ...]      upload a project folder (all runs, or the named ones)
  store.py ls <prefix>                    list what is there

Reads ~/.config/cf/r2 (access_key_id, secret_access_key, endpoint, bucket). Needs boto3, kept in
a private Python environment and not a dependency of the site. Text and raw-float files are
gzipped before upload and stored with their content encoding, so a browser gets them small;
snapshots.json (the engine's saved states, which no reader needs) is left behind.
"""
import os, sys, gzip, mimetypes, boto3
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
conf = dict(l.strip().split('=', 1) for l in open(os.path.expanduser('~/.config/cf/r2')) if '=' in l and not l.startswith('#'))
s3 = boto3.client('s3', endpoint_url=conf['endpoint'], aws_access_key_id=conf['access_key_id'], aws_secret_access_key=conf['secret_access_key'], region_name='auto')
BUCKET = conf.get('bucket', 'esimv1')
TYPES = {'.json': 'application/json', '.jsonl': 'application/x-ndjson', '.toml': 'application/toml; charset=utf-8', '.f32': 'application/octet-stream',
         '.md': 'text/markdown; charset=utf-8', '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.geojson': 'application/geo+json', '.csv': 'text/csv; charset=utf-8'}
SQUEEZE = set(TYPES)

def put(path, key):
    ext = os.path.splitext(path)[1]; data = open(path, 'rb').read(); extra = {'ContentType': TYPES.get(ext) or mimetypes.guess_type(path)[0] or 'application/octet-stream'}
    raw = len(data)
    if ext in SQUEEZE and raw > 1024: data = gzip.compress(data, 6); extra['ContentEncoding'] = 'gzip'
    # a run folder under a name is replaced when its variant is rerun, so nothing here may be cached for long
    extra['CacheControl'] = 'public, max-age=60'
    s3.put_object(Bucket=BUCKET, Key=key, Body=data, **extra)
    return raw, len(data)

cmd = sys.argv[1] if len(sys.argv) > 1 else ''
if cmd == 'buckets': print([b['Name'] for b in s3.list_buckets()['Buckets']])
elif cmd == 'ls':
    n = size = 0
    for page in s3.get_paginator('list_objects_v2').paginate(Bucket=BUCKET, Prefix=sys.argv[2] if len(sys.argv) > 2 else ''):
        for o in page.get('Contents', []): n += 1; size += o['Size']
    print(f'{n} objects, {size / 1e6:.1f} MB')
elif cmd == 'put':
    proj, only = sys.argv[2], set(sys.argv[3:]); root = os.path.realpath(os.path.join(HERE, 'projects', proj)); todo = []
    for d, dirs, files in os.walk(root):
        rel = os.path.relpath(d, root); parts = rel.split(os.sep)
        if parts[0] == 'runs' and len(parts) > 1 and only and parts[1] not in only: dirs[:] = []; continue
        if parts[0] == 'data': dirs[:] = []; continue                      # the record is in the scenario's repository
        for f in files:
            if f == 'snapshots.json' or f.endswith('.tmp'): continue
            todo.append((os.path.join(d, f), f'projects/{proj}/' + os.path.relpath(os.path.join(d, f), root).replace(os.sep, '/')))
    with ThreadPoolExecutor(32) as pool: done = list(pool.map(lambda a: put(*a), todo))
    print(f'{len(done)} files, {sum(r for r, _ in done) / 1e6:.1f} MB on disk, {sum(s for _, s in done) / 1e6:.1f} MB stored, to {BUCKET}/projects/{proj}')
else: print(__doc__)
