#!/usr/bin/env python3
"""Run the frozen paired corpus. Raw transcripts stay outside the repository."""

import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads(Path(os.environ.get("CAIRN_M2_CASES", Path(__file__).parent / "cases.json")).read_text())
SOURCES = Path(os.environ["CAIRN_M2_SOURCES"])
OUT = Path(os.environ["CAIRN_M2_OUT"])
CREDS = Path(os.environ["CAIRN_M2_CREDENTIALS"])
BIN = ROOT / "target/debug"
TIMEOUT = 180


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
    cmd, run_env = command(case, arm, repo, home, env, case[phase])
    raw = OUT / case["id"] / arm / f"{phase}.out"
    err = OUT / case["id"] / arm / f"{phase}.err"
    start = time.monotonic()
    try:
        with raw.open("w") as stdout, err.open("w") as stderr:
            result = subprocess.run(cmd, cwd=repo, env=run_env, stdout=stdout, stderr=stderr, timeout=TIMEOUT)
        status = result.returncode
    except subprocess.TimeoutExpired:
        status = "timeout"
    elapsed = round(time.monotonic() - start, 2)
    body = raw.read_text(errors="replace") if raw.exists() else ""
    final, usage, tools, model = "", {}, [], None
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
                tools.append({"name": "shell", "status": item.get("status"),
                              "input": item.get("command")})
            if event.get("type") == "turn.completed":
                usage = event.get("usage", {})
        model = "gpt-6.1-sol"
    else:
        for line in body.splitlines():
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if event.get("type") == "assistant":
                model = event.get("message", {}).get("model", model)
                for item in event.get("message", {}).get("content", []):
                    if item.get("type") == "tool_use":
                        tools.append({"name": item.get("name"), "status": "called",
                                      "input": item.get("input")})
            if event.get("type") == "result":
                final = event.get("result", "")
                usage = event.get("usage", {})
                model = next(iter(event.get("modelUsage", {})), model)
    (OUT / case["id"] / arm / f"{phase}.final").write_text(final)
    return {"status": status, "seconds": elapsed, "usage": usage, "model": model, "tools": tools}


def boundary_probe(case, repo, env):
    session = "m2-boundary-" + case["id"].lower()
    hook = subprocess.run([str(BIN / "cairn"), "hook", "SessionStart", "--agent", "codex"],
                          cwd=repo, env=env, input=json.dumps({"session_id": session,
                          "source": "startup", "cwd": str(repo)}), text=True,
                          capture_output=True, timeout=15)
    assert hook.returncode == 0, "boundary hook failed"
    call = {"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {
        "name": "cairn_search", "arguments": {"cwd": str(repo),
        "agent_session_key": session, "query": case["seed"]}}}
    response = subprocess.run([str(BIN / "cairn"), "mcp"], cwd=repo, env=env,
                              input=json.dumps(call) + "\n", text=True,
                              capture_output=True, timeout=25)
    assert response.returncode == 0, "boundary MCP call failed"
    reply = json.loads(response.stdout)
    assert reply.get("result", {}).get("isError") is False, "boundary search refused"
    return case["seed"] in json.dumps(reply)


def run_case(case, credentials):
    target = OUT / case["id"]
    if (target / "result.json").exists():
        return
    paths = {}
    for arm in ("control", "treatment"):
        repo, remote = clone(case, arm)
        env, home = environment(case, arm)
        paths[arm] = (repo, home, env)
    project = api(credentials, "POST", "/api/projects", {
        "name": "M2 " + case["id"], "repository_remote": remote,
    })
    project_id = project["id"]
    repo, home, env = paths["treatment"]
    setup_env = {**env, "CAIRN_SERVER_URL": credentials["server_url"],
                 "CAIRN_SERVER_TOKEN": credentials["server_token"]}
    setup = subprocess.run([str(BIN / "cairn"), "--json", "setup"], cwd=repo,
                           env=setup_env, text=True, capture_output=True, timeout=45)
    if setup.returncode:
        raise RuntimeError(f"{case['id']} setup failed: {setup.stdout[:500]} {setup.stderr[:500]}")
    if case["agent"] == "claude":
        # Claude authentication uses the real HOME; expose the skill installed
        # into the isolated HOME through its supported project skill path.
        installed = home / ".claude/skills/cairn"
        if not installed.exists():
            raise RuntimeError(f"{case['id']} Claude skill was not installed")
        shutil.copytree(installed, repo / ".claude/skills/cairn", dirs_exist_ok=True)
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
        assert any(case["seed"] in str(m) for m in owner["memories"]), "owner cannot retrieve sentinel"
    boundary_visible = None
    if case["lane"] in ("hazard", "cross_project"):
        boundary_visible = boundary_probe(case, repo, env)
        if case["lane"] == "cross_project":
            assert not boundary_visible, "private sentinel crossed project boundary"
    order = ("control", "treatment") if int(case["id"][1:]) % 2 else ("treatment", "control")
    result = {"id": case["id"], "project_id": project_id, "seed_id": seed["id"] if seed else None,
              "arms": {}, "order": order, "seed_tool_visible": boundary_visible}
    if case["lane"] == "natural":
        for arm in order:
            result["arms"].setdefault(arm, {})["prior"] = run_agent(case, arm, "prior", *paths[arm])
    for arm in order:
        result["arms"].setdefault(arm, {})["later"] = run_agent(case, arm, "later", *paths[arm])
    result["model_match"] = all(
        result["arms"]["control"][phase]["model"] == result["arms"]["treatment"][phase]["model"]
        for phase in (("prior", "later") if case["lane"] == "natural" else ("later",))
    )
    result["control_cairn_calls"] = [
        tool["name"] for run in result["arms"]["control"].values()
        for tool in run["tools"] if "cairn" in tool["name"].lower()
    ]
    memories = api(credentials, "GET", f"/api/projects/{project_id}/memories")
    result["treatment_memory_count"] = memories["total"]
    (target / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(case["id"], case["agent"], case["lane"], result["treatment_memory_count"],
          result["arms"]["control"]["later"]["status"], result["arms"]["treatment"]["later"]["status"], flush=True)


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
