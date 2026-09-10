"""Read a Castflow DuckDB file without initializing or modifying its schema."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path

import duckdb

QUERY = """SELECT e.id, e.title, p.title, e.url, CAST(e.date AS DATE),
t.id, t.speaker, t.timestamp_start, t.timestamp_end, t.text
FROM episodes e JOIN podcasts p ON p.id=e.podcast_id
JOIN transcripts t ON t.episode_id=e.id
WHERE e.id IN ({slots}) ORDER BY e.id,t.timestamp_start,t.id LIMIT 10001"""


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()


def file_hash(path):
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def private_write(path, content):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, "w") as f:
        f.write(content)


def export(db, ids, out, segments=None):
    db, out = Path(db), Path(out)
    if not db.is_file() or out.exists():
        raise ValueError("Database must exist; output directory must be new")
    if not 1 <= len(ids) <= 20 or len(set(ids)) != len(ids) or any(type(x) is not int or x < 1 for x in ids):
        raise ValueError("Select 1..20 unique positive episode IDs")
    if segments is not None and (not segments or len(segments) > 1000):
        raise ValueError("Select 1..1000 segment IDs")
    before = file_hash(db)
    stat = db.stat()
    with duckdb.connect(str(db), read_only=True) as conn:
        conn.execute("BEGIN TRANSACTION")
        rows = conn.execute(QUERY.format(slots=",".join("?" for _ in ids)), ids).fetchall()
        conn.execute("COMMIT")
    if len(rows) > 10000:
        raise ValueError("Query exceeds 10000 segments; select fewer episodes")
    if file_hash(db) != before or db.stat().st_mtime_ns != stat.st_mtime_ns:
        raise ValueError("Database changed during export; use a stable snapshot")
    found = {r[0] for r in rows}
    if found != set(ids):
        raise ValueError(f"Missing episodes or transcripts: {set(ids)-found}")
    if segments is not None:
        rows = [r for r in rows if r[5] in segments]
        if {r[5] for r in rows} != set(segments):
            raise ValueError("Selected segment not found in selected episodes")
    if len(rows) > 1000:
        raise ValueError("Export exceeds 1000 evidence records; provide --segments")
    manifest = {"version": 1, "adapter": "castflow-readonly-v1", "database_sha256": before,
                "episode_ids": sorted(ids), "segment_ids": sorted(r[5] for r in rows),
                "query_sha256": digest(QUERY), "record_count": len(rows)}
    evidence = []
    for eid, title, source, uri, day, tid, speaker, start, end, raw in rows:
        # Missing or zero-width timestamps never become fabricated transcript evidence.
        valid = (start is not None and end is not None and math.isfinite(start)
                 and math.isfinite(end) and 0 <= start < end <= 86400
                 and round(start*1000) < round(end*1000))
        if not day or not raw or not uri:
            raise ValueError(f"Missing source metadata for segment {tid}")
        excerpt = " ".join(raw.split())[:180]
        evidence.append({"id": f"ev-{tid}", "episode_id": str(eid), "episode_title": title,
                         "source": source, "speaker": speaker or "Unattributed transcript speaker",
                         "uri": uri, "asserted_at": str(day), "kind": "transcript" if valid else "transcript_untimed",
                         "segment_id": str(tid), "start_ms": round(start*1000) if valid else None,
                         "end_ms": round(end*1000) if valid else None, "excerpt": excerpt,
                         "segment_digest": digest(raw), "export_digest": ""})
    manifest["evidence_digest"] = digest(evidence)
    export_hash = digest(manifest)
    for record in evidence:
        record["export_digest"] = export_hash
    out.mkdir(mode=0o700, parents=True)
    private_write(out / "manifest.json", json.dumps(manifest, indent=2))
    private_write(out / "evidence.jsonl", "".join(json.dumps(e, ensure_ascii=False)+"\n" for e in evidence))
    return manifest


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--db", required=True, type=Path)
    p.add_argument("--episodes", required=True, help="Comma-separated IDs")
    p.add_argument("--segments", help="Optional comma-separated transcript IDs")
    p.add_argument("--out", required=True, type=Path)
    a = p.parse_args()
    try:
        print(json.dumps(export(a.db, list(map(int, a.episodes.split(","))), a.out,
                                set(map(int, a.segments.split(","))) if a.segments else None), indent=2))
    except (ValueError, OSError, duckdb.Error) as e:
        p.exit(2, f"Export failed: {e}\n")


if __name__ == "__main__":
    main()
