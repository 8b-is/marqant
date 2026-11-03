#!/usr/bin/env python3
"""Serial, integer-timed development benchmarks; never overwrite an existing CSV.

Build salient_template_arena first. Run tune, then validate with --tuning CSV.
Validation freezes parameters from the tuning prefix and includes ablations.
No floating-point arithmetic is used in this measurement wrapper.
"""
import argparse
import csv
import io
import os
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("stage", choices=("tune", "validate"))
parser.add_argument("--corpus", default="/data/corpus/enwik8")
parser.add_argument("--binary", default="./target/release/salient_template_arena")
parser.add_argument("--output", required=True)
parser.add_argument("--tuning")
args = parser.parse_args()

# Tuples: max_templates, max_record, min_gain, context, lattice_literals.
if args.stage == "tune":
    configurations = [(count, 512, 64, context, lattice)
                      for count in (32, 128)
                      for context in (False, True)
                      for lattice in (False, True)]
    configurations.extend([(128, 4096, 64, context, True) for context in (False, True)])
    ranges = [("tuning", 0, 2_000_000)]
else:
    if os.stat(args.corpus).st_size < 52_000_000:
        parser.error("validation requires a corpus containing the complete 50–52 MB slice")
    if not args.tuning:
        parser.error("validate requires --tuning CSV")
    with open(args.tuning, newline="") as source:
        measurements = list(csv.DictReader(source))
    if not measurements or any(row["bit_exact"] != "true" for row in measurements):
        raise SystemExit("Incomplete or invalid tuning measurements")
    configurations = []
    for lattice in (False, True):
        candidates = [row for row in measurements
                      if int(row["lattice_literals"]) == int(lattice)
                      and int(row["max_templates"]) > 0]
        best = min(candidates, key=lambda row: (int(row["archive_bytes"]),
                   int(row["max_templates"]), int(row["max_record"]), int(row["context"])))
        configuration = (int(best["max_templates"]), int(best["max_record"]),
                         int(best["min_gain"]), bool(int(best["context"])), lattice)
        configurations.append(configuration)
    combined = configurations[-1]
    # Flip only context prediction: isolates the transition model.
    configurations.append((combined[0], combined[1], combined[2], not combined[3], True))
    # No dictionary: exposes the literal backend and framing overhead alone.
    configurations.append((0, combined[1], combined[2], False, True))
    configurations = list(dict.fromkeys(configurations))
    ranges = [("heldout", 50_000_000, 2_000_000), ("full", 0, None)]

with open(args.output, "x", newline="") as destination:
    writer = None
    for phase, offset, limit in ranges:
        for count, record_size, margin, context, lattice in configurations:
            command = [args.binary, "bench", args.corpus,
                       "--templates", str(count), "--record-bytes", str(record_size),
                       "--min-gain", str(margin)]
            if offset:
                command += ["--offset", str(offset)]
            if limit is not None:
                command += ["--limit-bytes", str(limit)]
            if not context:
                command.append("--no-context")
            if lattice:
                command.append("--lattice-literals")
            print("RUN", phase, " ".join(command), flush=True)
            started = time.monotonic_ns()
            child = subprocess.Popen(command, stdout=subprocess.PIPE, text=True)
            result = child.stdout.read()
            child.stdout.close()
            _, status, usage = os.wait4(child.pid, 0)
            child.returncode = os.waitstatus_to_exitcode(status)
            if child.returncode:
                raise SystemExit(child.returncode)
            rows = list(csv.DictReader(io.StringIO(result)))
            if len(rows) != 1 or rows[0].get("bit_exact") != "true":
                raise RuntimeError("Missing byte-exact measurement: " + result)
            row = rows[0]
            row.update(phase=phase, wall_ns=str(time.monotonic_ns()-started),
                       peak_rss_kib=str(usage.ru_maxrss))
            if writer is None:
                writer = csv.DictWriter(destination, fieldnames=list(row))
                writer.writeheader()
            writer.writerow(row)
            destination.flush()
            print("RESULT", dict(row), flush=True)
print("COMPLETE", args.output, flush=True)
