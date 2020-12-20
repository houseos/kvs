/*
*  Performance tests for kvs
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

// To run these tests use: `cargo test integration -- --test-threads=1`.
// The --test-threads=1 option is crucial, otherwise cargo tries to execute
// all tests in parallel that would result in kvsd instances trying to bind to the same port.
// Run the following to see println!():
// `cargo test integration -- --test-threads=1 --nocapture`
#[cfg(test)]
mod tests {

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
}
