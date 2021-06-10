#!/bin/bash

xs=(2048 4096 8192 16384)
ys=(256  256  256  256)

for i in {0..3}; do
    ./run.sh ${xs[${i}]} ${ys[${i}]}
done