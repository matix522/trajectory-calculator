#!/bin/bash

xs=(256 512 1024 2048)
ys=(256 512 1024 2048)

for i in {0..3}; do
    ./run.sh ${xs[${i}]} ${ys[${i}]}
done