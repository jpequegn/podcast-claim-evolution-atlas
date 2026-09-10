import json
import tempfile
import unittest
from pathlib import Path
import duckdb
from export_castflow import export, file_hash

class ExportTests(unittest.TestCase):
    def test_readonly_and_timestamp_honesty(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d); db=root/"test.db"
            with duckdb.connect(str(db)) as c:
                c.execute("create table podcasts(id int,title text); create table episodes(id int,title text,podcast_id int,url text,date date); create table transcripts(id int,episode_id int,speaker text,timestamp_start double,timestamp_end double,text text)")
                c.execute("insert into podcasts values(1,'Synthetic'); insert into episodes values(1,'Fixture',1,'https://example.org/1','2026-01-01')")
                c.execute("insert into transcripts values(1,1,'Fixture',1,2,'Test evidence'),(2,1,NULL,0,0,'No reliable timestamp')")
            before=file_hash(db)
            export(db,[1],root/"out")
            self.assertEqual(before,file_hash(db))
            rows=[json.loads(x) for x in (root/"out/evidence.jsonl").read_text().splitlines()]
            self.assertEqual(rows[0]["kind"],"summary_only")
            self.assertEqual(rows[1]["kind"],"transcript")
            with self.assertRaises(ValueError): export(db,[1],root/"out")
            with self.assertRaises(ValueError): export(db,[2],root/"missing")
            self.assertFalse((root/"missing").exists())
    def test_limits(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(ValueError): export(Path(d)/"absent",[1],Path(d)/"out")
if __name__=="__main__": unittest.main()

