#!/usr/bin/env python3
"""Run the frozen paired corpus. Raw transcripts stay outside the repository."""

import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import re
import shutil
import hashlib
import secrets
import subprocess
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads(Path(os.environ.get("CAIRN_M2_CASES", Path(__file__).parent / "cases.json")).read_text())
SOURCES = Path(os.environ["CAIRN_M2_SOURCES"])
OUT = Path(os.environ["CAIRN_M2_OUT"])
CREDS = Path(os.environ["CAIRN_M2_CREDENTIALS"])
BIN = ROOT / "target/debug"
TIMEOUT = 180
RAW_LIMIT = 2_000_000


def git_sha(path):
    """Return a source identity without copying any source content to results."""
    result = subprocess.run(["git", "rev-parse", "HEAD"], cwd=path,
                            text=True, capture_output=True)
    return result.stdout.strip() if result.returncode == 0 else None


def identities(case):
    corpus = Path(os.environ.get("CAIRN_M2_CASES", Path(__file__).parent / "cases.json"))
    source = SOURCES / case["repo"]
    return {
        "candidate_sha": git_sha(ROOT),
        "corpus_sha256": hashlib.sha256(corpus.read_bytes()).hexdigest(),
        "source_sha": git_sha(source),
    }


def configured_model(case):
    return "gpt-6.1-sol" if case["agent"] == "codex" else "claude-sonnet-5-5"


def model_status(arms, phases):
    """Only reject observed reported-model drift; an absent report stays unverified."""
    reported = []
    drift = False
    for phase in phases:
        pair = []
        for arm in ("control", "treatment"):
            run = arms[arm].get(phase, {})
            models = set(run.get("reported_models", []))
            if run.get("reported_model"):
                models.add(run["reported_model"])
            pair.append(models)
            reported.append(models)
        drift = drift or (bool(pair[0]) and bool(pair[1]) and pair[0] != pair[1])
    return {
        "reported_models": sorted(set().union(*reported)),
        "reported_identity_verified": all(reported),
        "reported_model_drift": drift,
    }


def inspection(returncode, stdout, sentinel, expect_error=None, hook=False):
    """Return bounded probe metadata; never retain tool or hook output."""
    try:
        response = json.loads(stdout) if returncode == 0 else {}
    except json.JSONDecodeError:
        response = {}
    if hook:
        context = response.get("hookSpecificOutput", {}).get("additionalContext")
        json_valid = isinstance(context, str)
        context_status = ("reduced_or_unavailable" if json_valid and
                          ("Reduced context:" in context or "Fresh knowledge is unavailable" in context)
                          else "rendered_unverified")
        is_error = None
    else:
        json_valid = isinstance(response.get("result"), dict)
        context_status = None
        is_error = response.get("result", {}).get("isError")
    return {
        "process_ok": returncode == 0,
        "json_valid": json_valid,
        "expected_error": expect_error,
        "error": is_error is True,
        "sentinel_visible": sentinel in stdout,
        "sentinel_absent": sentinel not in stdout,
        # A rendered hook context is transport evidence only. It does not prove
        # a selected claim was captured or delivered, especially on fallback.
        "context_status": context_status,
        "passed": returncode == 0 and json_valid and (
            True if expect_error is None else is_error is expect_error),
    }


def mcp_call(repo, env, method, params):
    call = {"jsonrpc": "2.0", "id": 1, "method": method, "params": params}
    response = subprocess.run([str(BIN / "cairn"), "mcp"], cwd=repo, env=env,
                              input=json.dumps(call) + "\n", text=True,
                              capture_output=True, timeout=25)
    return response.returncode, response.stdout


def bound_private_raw(path):
    """Keep transcript inspection possible without allowing unbounded private storage."""
    if path.exists() and path.stat().st_size > RAW_LIMIT:
        with path.open("rb+") as output:
            output.truncate(RAW_LIMIT)
        return False
    return True


def api(credentials, method, path, body=None):
    request = urllib.request.Request(
        credentials["server_url"] + path,
        data=json.dumps(body).encode() if body is not None else None,
        headers={
            "Authorization": "Bearer " + credentials["server_token"],
            "Content-Type": "application/json",
        },
        method=method,
    )
    with urllib.request.urlopen(request, timeout=25) as response:
        return json.load(response)


def clone(case, arm):
    path = OUT / case["id"] / arm / "repo"
    path.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(["git", "clone", "-q", "--shared", str(SOURCES / case["repo"]), str(path)], check=True)
    remote = f"https://github.com/cairn-m2-eval/{OUT.name}-{case['id'].lower()}.git"
    subprocess.run(["git", "remote", "set-url", "origin", remote], cwd=path, check=True)
    if arm == "control" and case["repo"] == "Cairn":
        for name in ("AGENTS.md", "CLAUDE.md"):
            file = path / name
            file.write_text(re.sub(
                r"<!-- cairn:managed:begin.*?<!-- cairn:managed:end[^\n]*\n?",
                "", file.read_text(), flags=re.S,
            ))
        (path / ".claude/settings.json").unlink()
    return path, remote


def environment(case, arm):
    home = OUT / case["id"] / arm / "home"
    (home / ".codex").mkdir(parents=True, exist_ok=True)
    (home / ".claude").mkdir(exist_ok=True)
    auth = Path.home() / ".codex/auth.json"
    link = home / ".codex/auth.json"
    if not link.exists():
        link.symlink_to(auth)
    env = dict(os.environ)
    env.update({
        "HOME": str(home),
        "XDG_CONFIG_HOME": str(home / ".config"),
        "CODEX_HOME": str(home / ".codex"),
        "CAIRN_HOME": str(home / "cairn"),
        "CAIRND_BIN": str(BIN / "cairnd"),
        "PATH": str(BIN) + os.pathsep + os.environ["PATH"],
    })
    if arm == "treatment":
        wrapper = home / "capture-bin"
        wrapper.mkdir(exist_ok=True)
        link = wrapper / "cairn"
        if not link.exists():
            link.symlink_to(ROOT / "evals/m1-m2/hook_capture.py")
        env["PATH"] = str(wrapper) + os.pathsep + env["PATH"]
        env["CAIRN_M2_REAL_BIN"] = str(BIN / "cairn")
    for key in ("CAIRN_SERVER_URL", "CAIRN_SERVER_TOKEN", "CAIRN_ACCOUNT_ID"):
        env.pop(key, None)
    return env, home


def command(case, arm, repo, home, env, prompt):
    prompt = "Do not edit files. Answer from this checkout.\n\n" + prompt
    if case["agent"] == "codex":
        return [
            "codex", "exec", "-C", str(repo), "-m", "gpt-6.1-sol",
            "--json", "--dangerously-bypass-approvals-and-sandbox",
            "--dangerously-bypass-hook-trust", prompt,
        ], env
    mcp = home / ".claude.json"
    if arm == "control":
        mcp = home / "empty-mcp.json"
        mcp.write_text('{"mcpServers":{}}')
    env = {**env, "HOME": str(Path.home())}
    return [
        "claude", "-p", "--model", "claude-sonnet-5-5",
        "--output-format", "stream-json", "--verbose", "--max-turns", "8",
        "--dangerously-skip-permissions", "--setting-sources", "project,local",
        "--strict-mcp-config", "--mcp-config", str(mcp), "--", prompt,
    ], env


def run_agent(case, arm, phase, repo, home, env):
    hook_log = OUT / case["id"] / arm / f"{phase}.hooks.jsonl"
    if arm == "treatment":
        env = {**env, "CAIRN_M2_HOOK_LOG": str(hook_log)}
    cmd, run_env = command(case, arm, repo, home, env, case[phase])
    raw = OUT / case["id"] / arm / f"{phase}.out"
    err = OUT / case["id"] / arm / f"{phase}.err"
    start = time.monotonic()
    started_at = time.time()
    try:
        with raw.open("w") as stdout, err.open("w") as stderr:
            result = subprocess.run(cmd, cwd=repo, env=run_env, stdout=stdout, stderr=stderr, timeout=TIMEOUT)
        status = result.returncode
    except subprocess.TimeoutExpired:
        status = "timeout"
    elapsed = round(time.monotonic() - start, 2)
    body = raw.read_text(errors="replace") if raw.exists() else ""
    final, usage, tools, reported_model = "", {}, [], None
    reported_models = set()
    if case["agent"] == "codex":
        for line in body.splitlines():
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            item = event.get("item", {})
            if item.get("type") == "agent_message":
                final = item.get("text", final)
            if item.get("type") == "mcp_tool_call" and event.get("type") != "item.started":
                tools.append({"name": f"{item.get('server')}.{item.get('tool')}",
                              "status": item.get("status")})
            if item.get("type") == "command_execution" and event.get("type") != "item.started":
                tools.append({"name": "shell", "status": item.get("status")})
            if event.get("type") == "turn.completed":
                usage = event.get("usage", {})
    else:
        for line in body.splitlines():
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if event.get("type") == "assistant":
                reported_model = event.get("message", {}).get("model", reported_model)
                if reported_model:
                    reported_models.add(reported_model)
                for item in event.get("message", {}).get("content", []):
                    if item.get("type") == "tool_use":
                        tools.append({"name": item.get("name"), "status": "called"})
            if event.get("type") == "result":
                final = event.get("result", "")
                usage = event.get("usage", {})
                reported_model = next(iter(event.get("modelUsage", {})), reported_model)
                reported_models.update(event.get("modelUsage", {}))
    final_path = OUT / case["id"] / arm / f"{phase}.final"
    final_path.write_text(final)
    trace_complete = all([bound_private_raw(path) for path in (raw, err, final_path, hook_log)])
    hook_records = []
    if hook_log.exists():
        for line in hook_log.read_text().splitlines():
            try:
                hook_records.append(json.loads(line))
            except json.JSONDecodeError:
                trace_complete = False
    trace_complete = trace_complete and all(record["complete"] and record["stdout_forwarded"] and record["stderr_forwarded"]
                                            for record in hook_records)
    return {"status": status, "seconds": elapsed, "usage": usage,
            "started_at_unix": started_at, "ended_at_unix": time.time(),
            "configured_model": configured_model(case), "reported_model": reported_model,
            "reported_models": sorted(reported_models),
            # Compatibility for summarize.py. This is deliberately not a configured model.
            "model": reported_model,
            "trace_complete": trace_complete,
            "hook_capture": {"enabled": arm == "treatment", "records": len(hook_records)},
            "tools": tools}


def boundary_probe(case, repo, env, private_memory_id=None):
    """Inspect every supported delivery surface without saving its contents."""
    session = "m2-boundary-" + case["id"].lower()
    hook = subprocess.run([str(BIN / "cairn"), "hook", "SessionStart", "--agent", "codex"],
                          cwd=repo, env=env, input=json.dumps({"session_id": session,
                          "source": "startup", "cwd": str(repo)}), text=True,
                          capture_output=True, timeout=15)
    probes = {
        "hook_context": inspection(hook.returncode, hook.stdout, case["seed"], hook=True),
    }
    for name, arguments, expect_error in (
        ("context", {"name": "cairn_context", "arguments": {"cwd": str(repo),
         "agent_session_key": session}}, False),
        ("search", {"name": "cairn_search", "arguments": {"cwd": str(repo),
         "agent_session_key": session, "query": case["seed"]}}, False),
        # This is argument validation; authorization is checked separately below.
        ("refusal", {"name": "cairn_search", "arguments": {"cwd": str(repo),
         "action": "graph"}}, True),
    ):
        code, stdout = mcp_call(repo, env, "tools/call", arguments)
        probes[name] = inspection(code, stdout, case["seed"], expect_error)
    code, stdout = mcp_call(repo, env, "tools/list", {})
    probes["metadata"] = inspection(code, stdout, case["seed"])
    if private_memory_id:
        code, stdout = mcp_call(repo, env, "tools/call", {
            "name": "cairn_search", "arguments": {"cwd": str(repo), "action": "graph",
                                                     "memory_id": private_memory_id}})
        probe = inspection(code, stdout, case["seed"], False)
        try:
            text = json.loads(stdout)["result"]["content"][0]["text"]
            probe["foreign_edges_absent"] = json.loads(text).get("edges") == []
        except (KeyError, IndexError, TypeError, json.JSONDecodeError):
            probe["foreign_edges_absent"] = False
        probe["passed"] = probe["passed"] and probe["foreign_edges_absent"]
        probes["foreign_graph"] = probe
    return probes


def account_refusal_probe(credentials, case, project_id, memory_id):
    """A target-project member must not read a private-project record by UUID."""
    email = f"{OUT.name}-{case['id'].lower()}@example.test"
    member = api(credentials, "POST", "/api/admin/users", {"email": email, "display_name": "Privacy probe"})
    base = credentials["server_url"]

    def cookie_post(path, body, cookie=None):
        headers = {"Content-Type": "application/json"}
        if cookie:
            headers["Cookie"] = cookie
        request = urllib.request.Request(base + path, data=json.dumps(body).encode(),
                                         headers=headers, method="POST")
        with urllib.request.urlopen(request, timeout=25) as response:
            return json.load(response), response.headers.get("Set-Cookie", "").split(";", 1)[0]

    _, cookie = cookie_post("/api/auth/login", {"email": email, "password": member["temporary_password"]})
    password = secrets.token_urlsafe(24)
    cookie_post("/api/auth/password", {"new_password": password}, cookie)
    _, cookie = cookie_post("/api/auth/login", {"email": email, "password": password})
    token, _ = cookie_post("/api/tokens", {"name": "privacy-probe"}, cookie)
    api(credentials, "POST", f"/api/projects/{project_id}/members", {"user_id": member["id"]})
    limited = {"server_url": base, "server_token": token["token"]}
    api(limited, "GET", f"/api/projects/{project_id}/memories")  # positive membership witness
    request = urllib.request.Request(base + f"/api/memories/{memory_id}",
                                     headers={"Authorization": "Bearer " + token["token"]})
    try:
        with urllib.request.urlopen(request, timeout=25) as response:
            status, body = response.status, response.read(65536).decode(errors="replace")
    except urllib.error.HTTPError as error:
        status, body = error.code, error.read(65536).decode(errors="replace")
    visible = case["seed"] in body
    return {"status": status, "expected_status": 404, "sentinel_visible": visible,
            "sentinel_absent": not visible, "passed": status == 404 and not visible}


def control_configuration(repo, home):
    """Check the only files that can configure Cairn hooks or its managed prompt."""
    files = {
        "codex_config": home / ".codex/config.toml",
        "codex_hooks": repo / ".codex/hooks.json",
        "claude_settings": repo / ".claude/settings.json",
        "claude_settings_local": repo / ".claude/settings.local.json",
        "claude_mcp": home / ".claude.json",
        "agents": repo / "AGENTS.md",
        "claude": repo / "CLAUDE.md",
    }
    mentions = []
    for name, path in files.items():
        if not path.exists():
            continue
        text = path.read_text(errors="replace")
        if name == "codex_config":
            # Codex writes a project trust entry; its path is not hook configuration.
            text = text.replace(str(repo), "").replace(str(repo.resolve()), "")
        if "cairn" in text.lower():
            mentions.append(name)
    return {"inspected": list(files), "cairn_mentions": mentions,
            "hook_configuration_absent": not any(name in mentions for name in
                                                   ("codex_config", "codex_hooks", "claude_settings",
                                                    "claude_settings_local", "claude_mcp")),
            "managed_instruction_absent": not any(name in mentions for name in ("agents", "claude"))}


def failed_result(case, stage, error, result=None):
    result = result or {"id": case["id"], "identities": identities(case),
                        "arms": {"control": {}, "treatment": {}}, "order": None,
                        "treatment_memory_count": None, "seed_tool_visible": None}
    result["failure"] = {"stage": stage, "kind": type(error).__name__}
    result["valid"] = False
    result.setdefault("invalid_reasons", []).append("runner_" + stage + "_failure")
    return result


def run_case(case, credentials):
    target = OUT / case["id"]
    if (target / "result.json").exists():
        return
    target.mkdir(parents=True, exist_ok=True)
    result = {"id": case["id"], "identities": identities(case),
              "arms": {"control": {}, "treatment": {}}, "order": None,
              "seed_tool_visible": None, "treatment_memory_count": None,
              "invalid_reasons": []}
    stage = "clone"
    try:
        paths = {}
        for arm in ("control", "treatment"):
            repo, remote = clone(case, arm)
            env, home = environment(case, arm)
            paths[arm] = (repo, home, env)
        stage = "project"
        project = api(credentials, "POST", "/api/projects", {
            "name": "M2 " + case["id"], "repository_remote": remote,
        })
        project_id = project["id"]
        result["project_id"] = project_id
        repo, home, env = paths["treatment"]
        stage = "setup"
        setup_env = {**env, "CAIRN_SERVER_URL": credentials["server_url"],
                     "CAIRN_SERVER_TOKEN": credentials["server_token"]}
        setup = subprocess.run([str(BIN / "cairn"), "--json", "setup"], cwd=repo,
                               env=setup_env, text=True, capture_output=True, timeout=45)
        if setup.returncode:
            raise RuntimeError("setup returned nonzero")
        if case["agent"] == "claude":
            installed = home / ".claude/skills/cairn"
            if not installed.exists():
                raise RuntimeError("Claude skill was not installed")
            shutil.copytree(installed, repo / ".claude/skills/cairn", dirs_exist_ok=True)
        stage = "seed"
        seed = None
        if case["lane"] == "hazard":
            seed = api(credentials, "POST", f"/api/projects/{project_id}/memories", {
                "type": "fact", "scope": "project", "topic_key": "eval." + case["id"].lower(),
                "value_key": "stale_claim", "content": case["seed"],
            })
        if case["lane"] == "cross_project":
            secret = api(credentials, "POST", "/api/projects", {
                "name": "M2 secret " + case["id"],
                "repository_remote": f"https://github.com/cairn-m2-eval/{OUT.name}-secret-{case['id'].lower()}.git",
            })
            seed = api(credentials, "POST", f"/api/projects/{secret['id']}/memories", {
                "type": "fact", "scope": "project", "topic_key": "eval." + case["id"].lower(),
                "value_key": "private_sentinel", "content": case["seed"],
            })
            owner = api(credentials, "GET", f"/api/projects/{secret['id']}/memories")
            if not any(case["seed"] in str(m) for m in owner["memories"]):
                raise RuntimeError("owner cannot retrieve sentinel")
        result["seed_id"] = seed["id"] if seed else None
        if case["lane"] in ("hazard", "cross_project"):
            stage = "boundary_probe"
            result["delivery_probe"] = boundary_probe(
                case, repo, env, seed["id"] if case["lane"] == "cross_project" else None)
            if case["lane"] == "cross_project":
                result["delivery_probe"]["account_refusal"] = account_refusal_probe(
                    credentials, case, project_id, seed["id"])
            result["seed_tool_visible"] = result["delivery_probe"]["search"]["sentinel_visible"]
            if not all(probe["passed"] for probe in result["delivery_probe"].values()):
                result["invalid_reasons"].append("delivery_probe_failure")
            if case["lane"] == "cross_project":
                result["privacy_probe_passed"] = all(
                    probe["sentinel_absent"] for probe in result["delivery_probe"].values())
                if not result["privacy_probe_passed"]:
                    result["invalid_reasons"].append("privacy_sentinel_visible")
            else:
                # A stale seed is expected to be visible somewhere; this is a
                # delivery observation, not a privacy assertion.
                result["stale_delivery"] = {
                    "measured": True,
                    "delivered": any(probe["sentinel_visible"]
                                     for probe in result["delivery_probe"].values()),
                    "surfaces": [name for name, probe in result["delivery_probe"].items()
                                 if probe["sentinel_visible"]],
                }
        order = ("control", "treatment") if int(case["id"][1:]) % 2 else ("treatment", "control")
        result["order"] = order
        stage = "agents"
        if case["lane"] == "natural":
            for arm in order:
                result["arms"][arm]["prior"] = run_agent(case, arm, "prior", *paths[arm])
            # The runner retains only bounded raw traces and cannot safely turn
            # their rendered text into a claim denominator. Semantic scoring
            # must label actual later deliveries from the protected traces.
            result["natural_delivery"] = {"measured": False,
                                          "reason": "requires protected-trace semantic scoring"}
        for arm in order:
            result["arms"][arm]["later"] = run_agent(case, arm, "later", *paths[arm])
        result["control_configuration"] = control_configuration(*paths["control"][:2])
        phases = ("prior", "later") if case["lane"] == "natural" else ("later",)
        for arm in order:
            for phase in phases:
                if result["arms"][arm][phase]["status"] != 0:
                    result["invalid_reasons"].append(f"agent_run_failure:{arm}:{phase}")
        result["model_identity"] = model_status(result["arms"], phases)
        if result["model_identity"]["reported_model_drift"]:
            result["invalid_reasons"].append("reported_model_drift")
        result["control_cairn_calls"] = [
            tool["name"] for run in result["arms"]["control"].values()
            for tool in run["tools"] if "cairn" in tool["name"].lower()
        ]
        if result["control_cairn_calls"]:
            result["invalid_reasons"].append("control_cairn_calls")
        if not result["control_configuration"]["hook_configuration_absent"]:
            result["invalid_reasons"].append("control_hook_configuration_present")
        if not result["control_configuration"]["managed_instruction_absent"]:
            result["invalid_reasons"].append("control_managed_instruction_present")
        stage = "memory_count"
        memories = api(credentials, "GET", f"/api/projects/{project_id}/memories?limit=100")
        result["treatment_memory_count"] = memories["total"]
        stage = "delivery_snapshot"
        traces = api(credentials, "GET", f"/api/projects/{project_id}/retrieval-traces?limit=100")
        details = [api(credentials, "GET", f"/api/retrieval-traces/{trace['trace_id']}")
                   for trace in traces["traces"]]
        snapshot = target / "delivery.private.json"
        snapshot.write_text(json.dumps({"memories": memories, "traces": details}, indent=2) + "\n")
        snapshot.chmod(0o600)
        result["delivery_snapshot"] = {
            "complete": memories["total"] < memories["limit"] and not traces.get("cursor"),
            "trace_count": len(details),
            "note": "Selected/transmitted server records are separate from CLI-observed consumption."}
    except Exception as error:
        result = failed_result(case, stage, error, result)
    result["valid"] = not result["invalid_reasons"]
    (target / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(case["id"], case["agent"], case["lane"], result["treatment_memory_count"],
          result["arms"]["control"].get("later", {}).get("status"),
          result["arms"]["treatment"].get("later", {}).get("status"), flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--case", help="Run one case id, otherwise all 30")
    parser.add_argument("--jobs", type=int, default=1)
    args = parser.parse_args()
    credentials = json.loads(CREDS.read_text())
    selected = [case for case in CASES if args.case is None or case["id"] == args.case]
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        list(pool.map(lambda case: run_case(case, credentials), selected))


if __name__ == "__main__":
    main()
