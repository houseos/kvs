#!/bin/bash

# clean up store
rm -r ../target/release/store.json

# Initialize kvsd using kvsc with 10.000 entries.
echo "Initialize kvsd with 1000 entries"
for ((i=0;i<10000;i++))
do
  ../target/release/kvsc store --key "key$i" --value "$VALUE$i" > /dev/null 2>&1
done

# request random values and measure time
echo "Request random values"
for ((i=9900;i<9910;i++))
do
  START_TIME=$(date +%s.%N)
  ../target/release/kvsc get --key "key$i"
  END_TIME=$(date +%s.%N)
  TIME_DIFF=$(echo "$END_TIME - $START_TIME" | bc)
  echo "---- Get key$i ----"
  echo "$TIME_DIFF"
done