#!/bin/bash

TIMES=$1
TEST_NAME=$2

echo "run test ${TIMES} times"

for ((ITER = 1; ITER < TIMES+1; ITER++))
do
	echo "iteration - ${ITER}"
	cargo test $TEST_NAME
done
