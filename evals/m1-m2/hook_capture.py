#!/usr/bin/env python3
"""Evaluation-only PATH wrapper: forward exact hook bytes and retain bounded evidence."""

import json
import os
from pathlib import Path
import subprocess
import sys
import threading

LIMIT = 2_000_000


def main():
    real = os.environ["CAIRN_M2_REAL_BIN"]
    log = os.environ.get("CAIRN_M2_HOOK_LOG")
    if sys.argv[1:2] != ["hook"] or not log:
        os.execv(real, [real, *sys.argv[1:]])
    process = subprocess.Popen([real, *sys.argv[1:]], stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE)
    record = {"arguments": sys.argv[1:], "complete": True}

    def forward(source, destination, name):
        chunks, total, forwarded = [], 0, True
        while chunk := source.read1(65536):
            if total < LIMIT:
                chunks.append(chunk[:LIMIT - total])
            total += len(chunk)
            try:
                destination.write(chunk)
                destination.flush()
            except BrokenPipeError:
                forwarded = False
        record[name] = b"".join(chunks).decode("utf-8", errors="replace")
        record[name + "_forwarded"] = forwarded
        record[name + "_complete"] = total <= LIMIT

    errors = threading.Thread(target=forward, args=(process.stderr, sys.stderr.buffer, "stderr"))
    errors.start()
    forward(process.stdout, sys.stdout.buffer, "stdout")
    errors.join()
    record["status"] = process.wait()
    record["complete"] = record["stdout_complete"] and record["stderr_complete"]
    # One append keeps simultaneous hook records from interleaving. Phase log
    # size is capped by the runner; raw input is never recorded.
    descriptor = os.open(Path(log), os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    try:
        body = (json.dumps(record) + "\n").encode()
        if os.write(descriptor, body) != len(body):
            raise OSError("incomplete hook evidence write")
    finally:
        os.close(descriptor)
    raise SystemExit(record["status"])


if __name__ == "__main__":
    main()
