#!/usr/bin/env bash

#
# Copyright (c) 2024 Thomas Preindl
# MIT License (see LICENSE or https://mit-license.org)
#

host="http://127.0.0.1:3000"

zk_bp_filename=$(mktemp)
zk_b_filename=$(mktemp)
jq '.[0]' testCreateBPZKP.json | curl -fs -X POST "${host}/create?snark=false&zktype=BuildingPart" -H "Content-Type: application/json" -d @- > "$zk_bp_filename"

if [ $? -ne 0 ]; then
  echo "Error creating ZK_BuildingPartEPD"
  rm "$zk_bp_filename" "$zk_b_filename"
  exit 1
fi

echo "Created building part EPD:"
jq 'del(..|.zkp?)' "$zk_bp_filename"
#jq '.[3]' testCreateBPZKP.json

#jq --argjson dpp "$zk_bp_epd" '.[3]|..|select(.did? == "did1" or .did? == "did2")|..|.credentialSubject?|objects' testCreateBPZKP.json  . += {"ddp_vp": $dpp}
jq '.[3]' testCreateBPZKP.json | jq --slurpfile dpp "$zk_bp_filename" '.buildingPartDpps.[].dpp_vp.verifiableCredential.[].credentialSubject += $dpp[0]' > "$zk_b_filename"
response=$(curl -fs -X POST "${host}/create?snark=false&zktype=Building" -H "Content-Type: application/json" -d "@$zk_b_filename")

if [ $? -ne 0 ]; then
  echo "Error creating ZK_BuildingEPD"
  rm "$zk_bp_filename" "$zk_b_filename"
  exit 1
fi

echo "Created building EPD:"
echo "$response" | jq 'del(..|.zkp?)'
# Clean up zk_bp_filename
rm "$zk_bp_filename" "$zk_b_filename"