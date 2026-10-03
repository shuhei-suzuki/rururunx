"""Measure warm CLI startup wall time (not agent or scheduler overhead)."""
import argparse
import json
import platform
import statistics
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("binary")
parser.add_argument("--runs", type=int, default=30)
args = parser.parse_args()
if args.runs < 1:
    parser.error("--runs must be positive")
subprocess.run([args.binary, "--version"], check=True, stdout=subprocess.DEVNULL)
samples = []
for _ in range(args.runs):
    start = time.perf_counter()
    subprocess.run([args.binary, "--help"], check=True, stdout=subprocess.DEVNULL)
    samples.append((time.perf_counter() - start) * 1000)
print(json.dumps({"command": "rrx --help", "platform": platform.platform(),
                  "runs": args.runs, "median_ms": statistics.median(samples),
                  "min_ms": min(samples), "max_ms": max(samples)}, indent=2))
