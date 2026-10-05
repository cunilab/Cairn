#!/usr/bin/env python3
"""Print bounded, content-free metrics for a paired run."""

import json
import os
from pathlib import Path
import statistics

cases = json.loads(Path(os.environ["CAIRN_M2_CASES"]).read_text())
out = Path(os.environ["CAIRN_M2_OUT"])
rows = []
for case in cases:
    result_path = out / case["id"] / "result.json"
    if not result_path.exists():
        rows.append({"id": case["id"], "missing": True})
        continue
    result = json.loads(result_path.read_text())
    row = {"id": case["id"], "lane": case["lane"],
           "memory_count": result["treatment_memory_count"],
           "boundary_visible": result["seed_tool_visible"]}
    for arm in ("control", "treatment"):
        for phase in ("prior", "later"):
            if phase not in result["arms"][arm]:
                continue
            run = result["arms"][arm][phase]
            names = [tool["name"] for tool in run["tools"]]
            prefix = arm + "_" + phase + "_"
            row[prefix + "status"] = run["status"]
            row[prefix + "seconds"] = run["seconds"]
            row[prefix + "model"] = run["model"]
            row[prefix + "remember_calls"] = sum("cairn_remember" in n for n in names)
            row[prefix + "search_calls"] = sum("cairn_search" in n for n in names)
            row[prefix + "read_calls"] = sum(n in ("shell", "Bash", "Read", "Grep", "Glob") for n in names)
            row[prefix + "input_tokens"] = run["usage"].get("input_tokens")
            row[prefix + "output_tokens"] = run["usage"].get("output_tokens")
    if case["lane"] == "cross_project":
        row["sentinel_in_agent_trace"] = any(
            case["seed"] in (out / case["id"] / arm / "later.out").read_text(errors="replace")
            for arm in ("control", "treatment")
        )
    rows.append(row)

natural = [row for row in rows if row.get("lane") == "natural"]
privacy = [row for row in rows if row.get("lane") == "cross_project"]
paired_reads = [(row["control_later_read_calls"], row["treatment_later_read_calls"])
                for row in rows if "control_later_read_calls" in row]
summary = {
    "cases": len(rows),
    "completed": sum(not row.get("missing", False) for row in rows),
    "natural_with_explicit_remember": sum(row.get("treatment_prior_remember_calls", 0) > 0 for row in natural),
    "natural_with_any_memory": sum(row.get("memory_count", 0) > 0 for row in natural),
    "later_nonzero_exit": sum(row.get(arm + "_later_status") != 0 for row in rows for arm in ("control", "treatment")),
    "privacy_boundary_leaks": sum(row.get("boundary_visible") is True for row in privacy),
    "privacy_trace_leaks": sum(row.get("sentinel_in_agent_trace") is True for row in privacy),
    "median_later_read_calls": {
        "control": statistics.median(pair[0] for pair in paired_reads),
        "treatment": statistics.median(pair[1] for pair in paired_reads),
    },
}
print(json.dumps({"summary": summary, "cases": rows}, indent=2))
