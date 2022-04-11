#!/bin/bash

for test in test-*.sh
do
    . ./$test
done

echo $FAILURES failures
