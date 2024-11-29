#!/usr/bin/bash

#
# Copyright (c) 2024 Thomas Preindl
# MIT License (see LICENSE or https://mit-license.org)
#

set -eoux

# Check if the first argument is "--version"
# n0vm checks if docker is available using this call
# see: https://github.com/risc0/risc0/blob/79de616506543634cb5d75b9db7f3aee3640d68c/risc0/groth16/src/docker.rs#L78
if [ "$1" == "--version" ]; then
    # Return without error
    exit 0
fi

# we expect parameters for a docker run command such as:
# docker run --rm -v /temp_dir:/mnt risczero/risc0-groth16-prover:v2024-05-17.1
# See r0vm code: https://github.com/risc0/risc0/blob/79de616506543634cb5d75b9db7f3aee3640d68c/risc0/groth16/src/docker.rs#L56
# Extract the volume temp dir where the files are stored
temp_dir="${4%%:*}"
input_path="$temp_dir/input.json"
proof_path="$temp_dir/proof.json"
public_path="$temp_dir/public.json"

# Run the prover
ulimit -s unlimited
./stark_verify "$input_path" output.wtns
rapidsnark stark_verify_final.zkey output.wtns "$proof_path" "$public_path"