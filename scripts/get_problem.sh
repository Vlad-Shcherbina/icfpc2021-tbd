#!/usr/bin/env bash 

# Usage:
# scripts/get_problem.sh {first..last}

project_root="$(dirname $0 )/../"

for n in "$@"
do
	wget https://poses.live/problems/$n/download -O $project_root/data/problems/$n.problem
done
