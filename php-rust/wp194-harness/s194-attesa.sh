#!/bin/bash
# waits for an outcome file (arg 1), at most arg 2 seconds
n=0; until [ -f "$1" ] || [ $n -ge "$2" ]; do sleep 10; n=$((n+10)); done; [ -f "$1" ]
