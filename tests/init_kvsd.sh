#!/bin/bash

# clean up release directory
rm -r ../target/release/store.json

# build release version
cargo build --release

# run kvsd
../target/release/kvsd &
kvs_pid=$!
# Initialize kvsd using kvsc with 100 entries.

VALUE="0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789"

echo "Initialize kvsd with 100 entries"
for ((i=0;i<100;i++))
do
  ../target/release/kvsc store --key "key$i" --value "$VALUE$i" > /dev/null 2>&1
done

# request random values and measure time.
echo "Request random values"
for ((i=0;i<10;i++))
do
  START_TIME=$(date +%s.%N)
  ../target/release/kvsc get --key "key$i"
  END_TIME=$(date +%s.%N)
  TIME_DIFF=$(echo "$END_TIME - $START_TIME" | bc)
  echo "---- Get key$i ----"
  echo "$TIME_DIFF"
done


# Initialize kvsd using kvsc with 1000 entries.
echo "Initialize kvsd with 1000 entries"
for ((i=100;i<1000;i++))
do
  ../target/release/kvsc store --key "key$i" --value "$VALUE$i" > /dev/null 2>&1
done

# request random values and measure time
echo "Request random values"
for ((i=800;i<810;i++))
do
  START_TIME=$(date +%s.%N)
  ../target/release/kvsc get --key "key$i"
  END_TIME=$(date +%s.%N)
  TIME_DIFF=$(echo "$END_TIME - $START_TIME" | bc)
  echo "---- Get key$i ----"
  echo "$TIME_DIFF"
done

# Initialize kvsd using kvsc with 10.000 entries.
echo "Initialize kvsd with 10.000 entries"
for ((i=1000;i<10000;i++))
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

kill "$kvs_pid"