import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools/session"))
import session


class SessionTests(unittest.TestCase):
    def test_repo_sessions_validate_and_render_without_private_paths(self):
        files=sorted((ROOT/"sessions").glob("*.toml"));self.assertEqual(len(files),23)
        for p in files:
            d=session.load(p);self.assertEqual(session.validate(d,str(p)),[])
            out=session.render(d);self.assertIn(session.HEADER,out);self.assertNotIn("/home/",out);self.assertIn("{home}",out);self.assertIn("{state}",out);self.assertNotIn("{repo}",out)
            self.assertEqual(out,session.render(d))

    def test_validation_rejects_invalid_and_allows_find_name(self):
        d={"schema":1,"name":"bad","q":"Q17","issues":[36],"evidence_only":[37],"step":[{"id":"x","kind":"nope","who":"script","run":["ls","{state}/soak-*.txt"]},{"id":"x","kind":"cmd","who":"script"}]}
        e="\n".join(session.validate(d,"bad.toml"));self.assertIn("unknown q",e);self.assertIn("duplicate id",e);self.assertIn("unknown kind",e);self.assertIn("glob",e);self.assertIn("evidence_only",e)
        d["step"][0]["run"]=["find","{state}","-name","soak-*.txt"]
        self.assertNotIn("glob", "\n".join(session.validate(d,"bad.toml")))
        home = "/home/" + "x"
        d["step"][0]["run"]=["cat",home]
        self.assertIn("literal home path", "\n".join(session.validate(d,"bad.toml")))
        d={"schema":1,"name":"renamed","q":"Q0","issues":[36],"evidence_only":[36],"step":[{"id":"x","kind":"cmd","who":"script"}]}
        errors="\n".join(session.validate(d,"other.toml"));self.assertIn("name",errors);self.assertIn("vars.appid",errors);self.assertIn("cmd needs run",errors)

    def test_validation_rejects_unknown_placeholder_and_missing_fields(self):
        d={"schema":1,"name":"bad","q":"Q0","issues":[],"step":[{"id":"x","kind":"cmd","who":"script","run":["echo","{unknown}"]}]}
        e="\n".join(session.validate(d,"bad.toml"));self.assertIn("unknown placeholder",e)

    def test_run_script_verdicts_and_reports(self):
        with tempfile.TemporaryDirectory() as td:
            root=pathlib.Path(td); csv=root/"fps.csv";csv.write_text("os,cpu\nLinux,0\nfps,frametime\n60,16\n60,16\n")
            doc={"name":"trial","q":"Q0","issues":[],"step":[
                {"id":"ok","kind":"cmd","who":"script","run":[sys.executable,"-c","print('ok')"],"expect":"ok"},
                {"id":"bad-json","kind":"cmd","who":"script","run":[sys.executable,"-c","print('{\"ok\":true,\"data\":{\"ok\":false}}')"],"expect_json":{"/data/ok":True},"optional":True},
                {"id":"optional","kind":"cmd","who":"script","run":["false"],"optional":True},
                {"id":"gate","kind":"check","who":"you"}]}
            r=session.run(doc,{"gate":"s"},root,123);self.assertEqual(r["verdict"],"INCOMPLETE");self.assertEqual(r["steps"][0]["status"],"ok");self.assertEqual(r["steps"][1]["status"],"fail")
            self.assertIn("commit",r);self.assertIn("dirty",r);self.assertTrue(all("who" in x for x in r["steps"]))
            self.assertIn("## Verified",session.to_markdown(r));self.assertIn("## Not verified",session.to_markdown(r))
            self.assertIn("Done when:",session.to_markdown({**r,"done_when":"fixture acceptance"}))
            with self.assertRaisesRegex(ValueError,"missing answer for step gate"): session.run(doc,{},root,124)

    def test_error_code_and_measure_results(self):
        with tempfile.TemporaryDirectory() as td:
            root=pathlib.Path(td);csv=root/"m.csv";csv.write_text("os,cpu\nLinux,0\nfps,frametime\n60,12\n60,20\n")
            doc={"name":"measures","q":"Q0","issues":[],"done_when":"measure fixture","step":[
                {"id":"code","kind":"cmd","who":"script","run":[sys.executable,"-c","print('{\"ok\":false,\"error\":{\"code\":\"link-not-ready\"}}')"],"expect_code":"link-not-ready"},
                {"id":"fps","kind":"measure","who":"you","tool":"fps","input":str(csv),"args":["--min","30"]},
                {"id":"optional","kind":"measure","who":"you","tool":"disparity","input":str(root/"missing-*.png"),"optional":True}]}
            result=session.run(doc,{},root,2);self.assertEqual(result["verdict"],"PASS");self.assertEqual(result["steps"][0]["code"],"link-not-ready");self.assertEqual(result["steps"][1]["metrics"]["p50"],60);self.assertEqual(result["steps"][2]["status"],"skip")

    def test_command_table_words_if_available(self):
        path=ROOT/"docs/code-map.md"
        if not path.exists() or "## Commands" not in path.read_text(encoding="utf-8"):
            print("SKIP command table check: ## Commands is not present yet");return
        text=path.read_text(encoding="utf-8");section=text.split("## Commands",1)[1].split("\n## ",1)[0]
        cells=set()
        for line in section.splitlines():
            if line.startswith("|") and not line.startswith("|---"):
                cell=line.split("|",2)[1].strip().strip("`");cells.add(cell.split()[0] if cell else "")
        for f in (ROOT/"sessions").glob("*.toml"):
            d=session.load(f)
            for s in d.get("step",[]):
                run=s.get("run",[])
                if len(run)>1 and run[0]=="flat3d-inject": self.assertIn(run[1].split()[0],cells,f"{f}:{s['id']}")

    def test_interactive_cli_prompts_and_does_not_run_owner_command(self):
        with tempfile.TemporaryDirectory() as td:
            root=pathlib.Path(td);toml=root/"interactive.toml";state=root/"state"
            toml.write_text('''schema = 1\nname = "interactive"\nq = "Q0"\nissues = []\ndone_when = "fixture acceptance"\n[[step]]\nid = "owner-action"\nkind = "cmd"\nwho = "you"\ntext = "Confirm this owner command."\nrun = ["false"]\n[[step]]\nid = "ask"\nkind = "ask"\nwho = "you"\ntext = "Enter a fixture value."\n[[step]]\nid = "gate"\nkind = "check"\nwho = "you"\ntext = "Approve fixture gate."\n''')
            cp=subprocess.run([sys.executable,str(ROOT/"tools/session/session.py"),"run",str(toml),"--state-dir",str(state),"--now","123"],input="y\nfixture-value\ny\n",capture_output=True,text=True)
            self.assertEqual(cp.returncode,0,cp.stderr+cp.stdout);self.assertIn("Confirm this owner command",cp.stdout);self.assertIn("false",cp.stdout)
            result=json.loads((state/"interactive-123.json").read_text());self.assertEqual(result["steps"][0]["status"],"ok");self.assertIsNone(result["steps"][0]["rc"]);self.assertEqual(result["steps"][1]["answer"],"fixture-value");self.assertIn("fixture acceptance",(state/"interactive-123.md").read_text())

    @unittest.skipIf(os.name=="nt","process group teardown is Unix-specific")
    def test_background_is_cleaned_when_script_answer_is_missing(self):
        with tempfile.TemporaryDirectory() as td:
            root=pathlib.Path(td);marker=root/"pid"
            doc={"name":"cleanup","q":"Q0","issues":[],"step":[{"id":"bg","kind":"cmd","who":"script","run":[sys.executable,"-c",f"import os,time;open({str(marker)!r},'w').write(str(os.getpid()));time.sleep(30)"],"background":True},{"id":"ask","kind":"ask","who":"you","text":"answer"}]}
            with self.assertRaisesRegex(ValueError,"missing answer"):session.run(doc,{},root,1)
            pid=int(marker.read_text())
            with self.assertRaises(ProcessLookupError):os.kill(pid,0)

    def test_evidence_never_says_close_and_pending_order(self):
        m=session.to_markdown({"session":"z5","verdict":"PASS","steps":[],"evidence_only":[36]})
        self.assertIn("Evidence for #36",m);self.assertNotRegex(m,r"(?i)\b(close[sd]?|fix(e[sd])?|resolve[sd]?)\s+#36\b")
        docs=[{"name":"g1-geo11","q":"Q9","after":["q0"]},{"name":"q0","q":"Q0"}]
        with tempfile.TemporaryDirectory() as td:
            pathlib.Path(td,"q0-1.json").write_text('{"session":"q0","verdict":"PASS"}')
            got=session.pending(docs,td);self.assertEqual([x["name"] for x in got],["g1-geo11"]);self.assertTrue(got[0]["ready"])

    def test_cli_dry_run(self):
        with tempfile.TemporaryDirectory() as td:
            bindir=pathlib.Path(td)
            # Fixed command inventory for the session files; dry-run only resolves names.
            for name in ("bash","grep","test","sha256sum","flat3d-inject","systemctl","adb","distrobox","python3","find","cargo","env","flatpak","unzip","chmod","false","mkdir"):
                p=bindir/name;p.write_text("#!/bin/sh\nexit 0\n");p.chmod(0o755)
            for p in (ROOT/"sessions").glob("*.toml"):
                cp=subprocess.run([sys.executable,str(ROOT/"tools/session/session.py"),"run",str(p),"--dry-run"],capture_output=True,text=True,env={**os.environ,"PATH":str(bindir)})
                self.assertEqual(cp.returncode,0,cp.stderr+cp.stdout)


if __name__=="__main__":unittest.main()
