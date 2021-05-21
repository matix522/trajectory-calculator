#!/bin/bash

cargo run --release -- -t naive  -o results/naive_${1}_${2}.tsv  -x ${1} -y ${2}
cargo run --release -- -t rc     -o results/rc_${1}_${2}.tsv     -x ${1} -y ${2}
cargo run --release -- -t rc+    -o results/rc+_${1}_${2}.tsv    -x ${1} -y ${2}
cargo run --release -- -t linear -o results/linear_${1}_${2}.tsv -x ${1} -y ${2}