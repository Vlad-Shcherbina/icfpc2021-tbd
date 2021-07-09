#! /usr/bin/env python3
import os.path
import subprocess

cargo = os.path.expanduser("~/.cargo/bin/cargo")

for i in range(1, 3):
    print("Start on {}".format(i))
    p = subprocess.Popen([cargo, "run", "--release", "solver", str(i)],
                         env={**os.environ,
                              "SOLVER": "random",
                              "DURATION_LIMIT_SECONDS": "5"})
    p.wait()