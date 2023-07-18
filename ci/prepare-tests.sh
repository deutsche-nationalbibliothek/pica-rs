#!/bin/bash

set -euo pipefail
# set -x

DIRS=$(find docs/book/src/referenz/kommandos -type f -iname "*.md" -print)
DUMP="tests/data/DUMP.dat.gz"

for i in $DIRS; do
    mkdir -p "${i%.md}.in"
    cp -v $DUMP "${i%.md}.in"
done
