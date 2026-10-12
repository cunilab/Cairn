#!/usr/bin/env python3
"""Collect full private judge packets without choosing favorable excerpts."""

from datetime import datetime
import json
import os
from pathlib import Path


def events(path):
    if not path.exists():
        return []
    records = []
    for index, line in enumerate(path.read_text(errors="replace").splitlines(), 1):
        try:
            record = {"event": json.loads(line)}
        except ValueError:
            record = {"unparsed": line}
        records.append({"line": index, **record})
    return records


def timestamp(value):
    return datetime.fromisoformat(value.replace("Z", "+00:00")).timestamp()


def collect(case, folder):
    result = json.loads((folder / "result.json").read_text())
    packet = {"case": case, "execution": result, "arms": {}}
    for arm, phases in result["arms"].items():
        packet["arms"][arm] = {}
        for phase, metadata in phases.items():
            prefix = folder / arm / phase
            context = prefix.with_suffix(".context.private.json")
            packet["arms"][arm][phase] = {
                "trace_complete": metadata["trace_complete"],
                "events": events(prefix.with_suffix(".out")),
                "hooks": events(prefix.with_suffix(".hooks.jsonl")),
                "delivered_context": json.loads(context.read_text()) if context.exists() else None,
                "stderr": prefix.with_suffix(".err").read_text(errors="replace"),
                "final": prefix.with_suffix(".final").read_text(errors="replace"),
            }
    snapshot = folder / "delivery.private.json"
    later = result["arms"].get("treatment", {}).get("later")
    if snapshot.exists() and later:
        delivery = json.loads(snapshot.read_text())
        memories = {memory["id"]: memory for memory in delivery["memories"]["memories"]}
        transmitted, selected = [], []
        for trace in sorted(delivery["traces"], key=lambda item: (item["created_at"], item["trace_id"])):
            if not later["started_at_unix"] <= timestamp(trace["created_at"]) <= later["ended_at_unix"]:
                continue
            for item in trace["items"]:
                if item["status"] != "selected":
                    continue
                memory = memories.get(item["knowledge_id"])
                entry = {"trace_id": trace["trace_id"], "created_at": trace["created_at"],
                         "delivery_state": trace["delivery_state"], "delivery_point": trace["delivery_point"],
                         "item": item, "current_record": memory,
                         "version_matches": memory is not None and
                         timestamp(memory["updated_at"]) == timestamp(item["source_updated_at"])}
                (transmitted if trace["delivery_state"] == "transmitted" else selected).append(entry)
        packet["server_later_delivery"] = {
            "snapshot_complete": result["delivery_snapshot"]["complete"],
            "transmitted_items": transmitted, "selected_without_transmission": selected,
            "qualifier": "Transmission proves delivery to the host, not consumption. "
                         "current_record is source evidence, not proof that its whole content was delivered. "
                         "An item's selection binds an excerpt to source bytes. Inventory claims from "
                         "the actual delivered context/tool payload; version mismatch cannot reconstruct it.",
        }
    return packet


def main():
    out = Path(os.environ["CAIRN_M2_OUT"])
    if out.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("Private packets must be outside the repository")
    packets = out / "packets"
    packets.mkdir(mode=0o700, exist_ok=True)
    token = json.loads(Path(os.environ["CAIRN_M2_CREDENTIALS"]).read_text())["server_token"]
    cases = json.loads(Path(os.environ["CAIRN_M2_CASES"]).read_text())
    for case in cases:
        folder = out / case["id"]
        target = packets / (case["id"] + ".json")
        if not (folder / "result.json").exists() or target.exists():
            continue
        text = json.dumps(collect(case, folder), ensure_ascii=True, indent=2) + "\n"
        target.write_text(text.replace(token, "[REDACTED API TOKEN]"))
        target.chmod(0o600)
        print(case["id"], target.stat().st_size, "bytes")


if __name__ == "__main__":
    main()
