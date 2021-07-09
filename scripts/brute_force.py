#! /usr/bin/env python3
import os.path
import subprocess

cargo = os.path.expanduser("~/.cargo/bin/cargo")

for i in range(1, 10):
    subprocess.run([cargo, "run", "--release", "random_solver", str(i)])