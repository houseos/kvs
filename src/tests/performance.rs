// ============== Load Tests ==============

// Test the maximum amount of entries in a JSON store

// Test the maximal size of a JSON store entry

// Test the performance of a JSON store with maximal size

// Test the file store with 10.000 entries

// Test the size boundary of the file store

// Test the performance of the file store with 10.000 entries (each 1 kilobyte)
/*
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
    */

// Test the performance of the file store with 10.000 entries (each 1 megabyte)

// Test the performance of the file store with 10.000 entries (each biggest size)
