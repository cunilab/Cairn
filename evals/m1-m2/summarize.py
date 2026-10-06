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
           "boundary_visible": result["seed_tool_visible"],
           "valid": result.get("valid"),
           "invalid_reasons": result.get("invalid_reasons", []),
           "failure": result.get("failure"),
           "delivery_probe": result.get("delivery_probe")}
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
            row[prefix + "configured_model"] = run.get("configured_model")
            row[prefix + "trace_complete"] = run.get("trace_complete")
            row[prefix + "remember_calls"] = sum("cairn_remember" in n for n in names)
            row[prefix + "search_calls"] = sum("cairn_search" in n for n in names)
            row[prefix + "read_calls"] = sum(n in ("shell", "Bash", "Read", "Grep", "Glob") for n in names)
            row[prefix + "input_tokens"] = run["usage"].get("input_tokens")
            row[prefix + "output_tokens"] = run["usage"].get("output_tokens")
    if case["lane"] == "cross_project":
        traces = [out / case["id"] / arm / ("later." + stream)
                  for arm in ("control", "treatment") for stream in ("out", "err")]
        row["sentinel_in_agent_trace"] = (any(case["seed"] in path.read_text(errors="replace")
                                            for path in traces)
                                           if all(path.exists() for path in traces) and
                                           all(row.get(arm + "_later_trace_complete") is True
                                               for arm in ("control", "treatment")) else None)
    rows.append(row)

natural = [row for row in rows if row.get("lane") == "natural"]
privacy = [row for row in rows if row.get("lane") == "cross_project"]
paired_reads = [(row["control_later_read_calls"], row["treatment_later_read_calls"])
                for row in rows if "control_later_read_calls" in row]
summary = {
    "cases": len(rows),
    "recorded": sum(not row.get("missing", False) for row in rows),
    "completed": sum(all(arm + "_later_status" in row for arm in ("control", "treatment"))
                     for row in rows),
    "valid_comparisons": sum(row.get("valid") is True for row in rows),
    "invalid_comparisons": sum(row.get("valid") is False for row in rows),
    "natural_with_explicit_remember": sum(row.get("treatment_prior_remember_calls", 0) > 0 for row in natural),
    "natural_with_any_memory": sum((row.get("memory_count") or 0) > 0 for row in natural),
    "runner_failures": sum(row.get("failure") is not None for row in rows),
    "later_nonzero_exit": sum(isinstance(row.get(arm + "_later_status"), int) and
                             row[arm + "_later_status"] != 0
                             for row in rows for arm in ("control", "treatment")),
    "later_timeouts": sum(row.get(arm + "_later_status") == "timeout"
                          for row in rows for arm in ("control", "treatment")),
    "privacy_boundary_leaks": sum(any(probe.get("sentinel_visible") is True
                                     for probe in row["delivery_probe"].values())
                                  if row.get("delivery_probe") is not None
                                  else row.get("boundary_visible") is True
                                  for row in privacy),
    "privacy_trace_leaks": sum(row.get("sentinel_in_agent_trace") is True for row in privacy),
    "privacy_probe_cases": sum(row.get("delivery_probe") is not None for row in privacy),
    "privacy_trace_cases": sum(row.get("sentinel_in_agent_trace") is not None for row in privacy),
    "median_later_read_calls": {
        "control": statistics.median(pair[0] for pair in paired_reads) if paired_reads else None,
        "treatment": statistics.median(pair[1] for pair in paired_reads) if paired_reads else None,
    },
}
print(json.dumps({"summary": summary, "cases": rows}, indent=2))
