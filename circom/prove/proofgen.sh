#!/bin/bash
set -e # Stop on error

SCRIPT_DIR=$(cd $(dirname $0); pwd)

gsutil cp gs://$BUCKET/$BLUEPRINT_ID/complete_circuit.zip ./complete_circuit.zip
unzip complete_circuit.zip

input_path="input.json"
witness_path="witness.wtns"
proof_path="proof.json"
public_path="public.json"

cd "${SCRIPT_DIR}"
echo "entered zk email path: ${SCRIPT_DIR}"

circuit "${input_path}" "${witness_path}" | tee /dev/stderr
status_jswitgen=$?
echo "✓ Finished witness generation with cpp! ${status_jswitgen}"


echo "ldd ${SCRIPT_DIR}/rapidsnark/package/bin/prover_cuda"
ldd "${SCRIPT_DIR}/rapidsnark/package/bin/prover_cuda"
status_lld=$?
echo "✓ lld prover dependencies present! ${status_lld}"

echo "${SCRIPT_DIR}/rapidsnark/package/bin/prover_cuda circuit.zkey ${witness_path} ${proof_path} ${public_path}"
"${SCRIPT_DIR}/rapidsnark/package/bin/prover_cuda" "circuit.zkey" "${witness_path}" "${proof_path}" "${public_path}"  | tee /dev/stderr
status_prover=$?
echo "✓ Finished rapid proofgen! Status: ${status_prover}"

exit 0