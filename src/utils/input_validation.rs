/*
*  input_validation utils module
*  SPDX-License-Identifier: MIT
*  Copyright (C) 2020 Benjamin Schilling
*/

// External crates
use regex::Regex;

// kvs modules
use crate::log::{log, LOG_STDERR};

// Constants
const KEY_LEN_MIN: usize = 1;
const KEY_LEN_MAX: usize = 32;
const VALUE_LEN_MIN: usize = 1;
const VALUE_LEN_MAX: usize = 1024;

pub fn validate_key(input: String) -> bool {
    lazy_static! {
        static ref RE_KEY: Regex = Regex::new(r"^\w*$").unwrap();
    }
    // Check length
    if input.len() < KEY_LEN_MIN || input.len() > KEY_LEN_MAX {
        return false;
    }
    // Check regex
    RE_KEY.is_match(&input)
}

pub fn validate_value(input: String, check_length: bool) -> bool {
    lazy_static! {
        // Allow only characters defined in Base64 alphabet (RFC4648)
        static ref RE_KEY: Regex = Regex::new(r"^[\w+/=]*$").unwrap();
    }
    // Check length
    if check_length && input.len() < VALUE_LEN_MIN || input.len() > VALUE_LEN_MAX {
        return false;
    }
    // Check regex
    RE_KEY.is_match(&input)
}

pub fn validate_path(input: String) -> bool {
    lazy_static! {
        // Allow only alphanumeric characters as well as "/", "\", ":" and "."
        static ref RE_KEY: Regex = Regex::new(r"^[\w/\\.:-]*$").unwrap();
    }
    // Check regex
    RE_KEY.is_match(&input)
}

pub fn validate_ipv4(input: String) -> bool {
    // Check IPv4 dotted notation
    lazy_static! {
        static ref RE_IP: Regex =
            Regex::new(r"^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$").unwrap();
    }
    // Shortes "0.0.0.0" = 7, longest "255.255.255.255" = 15
    if input.len() < 7 || input.len() > 15 {
        log("IP length invalid".to_string(), LOG_STDERR);
        return false;
    }
    //Check regex
    if !RE_IP.is_match(&input) {
        return false;
    }
    // Check range of all fields
    // Split string
    let octets: Vec<&str> = input.split('.').collect();
    // Check range from 0 to 255 of each octet
    for octet in octets {
        if octet.parse::<i16>().unwrap() < 0 || octet.parse::<u16>().unwrap() > 255 {
            return false;
        }
    }
    true
}

pub fn validate_port(input: String) -> bool {
    // Check for digits
    lazy_static! {
        static ref RE_IP: Regex = Regex::new(r"^(\d*)$").unwrap();
    }
    // Shortes "0" = 1, longest "65534" =
    if input.is_empty() || input.len() > 5 {
        log("Port length invalid".to_string(), LOG_STDERR);
        return false;
    }
    //Check regex
    if RE_IP.is_match(&input) {
        // Parse if only digits
        let value: i32 = input.parse().unwrap();
        // Check range from 0 to 65534
        value < 65535 && value > 0
    } else {
        false
    }
}

// To run these tests use: `cargo test input_validation`
// Run the following to see println!()
// cargo test input_validation_random_invalid -- --nocapture
#[cfg(test)]
mod tests {

    use super::*;
    use base64;
    use rand::Rng;

    // ============== Key Validation ===============================
    #[test]
    fn input_validation_key_ok() {
        assert_eq!(validate_key("test".to_string()), true)
    }
    #[test]
    fn input_validation_key_lower_length_boundary_ok() {
        assert_eq!(validate_key("1".to_string()), true)
    }
    #[test]
    fn input_validation_key_lower_length_boundary_failed() {
        assert_eq!(validate_key("".to_string()), false)
    }
    #[test]
    fn input_validation_key_upper_length_boundary_ok() {
        assert_eq!(
            validate_key("12345678901234567890123456789012".to_string()),
            true
        )
    }
    #[test]
    fn input_validation_key_upper_length_boundary_failed() {
        assert_eq!(
            validate_key("123456789012345678901234567890123".to_string()),
            false
        )
    }
    #[test]
    fn input_validation_key_random_string_repeated() {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();

        for _x in 0..10 {
            // println!("Key random string run: {}", x);
            // generate random length
            let string_len: usize = rng.gen_range(0, 50);
            // generate random string
            let test_string: String = (0..string_len)
                .map(|_| {
                    let idx = rng.gen_range(0, CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();
            // println!("Key string: {}", test_string.clone());
            if string_len < KEY_LEN_MIN || string_len > KEY_LEN_MAX {
                // If string length does not fit, expect fail
                assert_eq!(validate_key(test_string), false)
            } else {
                // If string length fits expect ok
                assert_eq!(validate_key(test_string), true)
            }
        }
    }
    // ============== Value Validation ===============================
    #[test]
    fn input_validation_value_lower_length_boundary_ok() {
        assert_eq!(validate_value("t".to_string(), true), true)
    }
    #[test]
    fn input_validation_value_lower_length_boundary_failed() {
        assert_eq!(validate_value("".to_string(), true), false)
    }
    #[test]
    fn input_validation_value_upper_length_boundary_ok() {
        let mut test: String = "".to_string();
        for _x in 0..VALUE_LEN_MAX {
            test = test + "1";
        }
        assert_eq!(validate_value(test, true), true)
    }
    #[test]
    fn input_validation_value_upper_length_boundary_failed() {
        let mut test: String = "".to_string();
        for _x in 0..VALUE_LEN_MAX + 1 {
            test = test + "1";
        }
        assert_eq!(validate_value(test, true), false)
    }
    #[test]
    fn input_validation_value_random_base64_string_repeated() {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();

        for x in 0..10 {
            println!("Base64 random string run: {}", x);
            // generate random length
            let string_len: usize = rng.gen_range(0, VALUE_LEN_MAX + 10);
            // generate random string
            let test_string: String = (0..string_len)
                .map(|_| {
                    let idx = rng.gen_range(0, CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();
            let base64_string: String = base64::encode(test_string.clone());
            let base64_string_len: usize = base64_string.len();
            println!(
                "Base64 string: {}, length: {}",
                base64_string.clone(),
                base64_string.len()
            );

            if base64_string_len < VALUE_LEN_MIN || base64_string_len > VALUE_LEN_MAX {
                // If string length does not fit, expect fail
                assert_eq!(validate_value(base64_string, true), false)
            } else {
                // If string length fits expect ok
                assert_eq!(validate_value(base64_string, true), true)
            }
        }
    }

    // ============== IP Validation ===============================
    #[test]
    fn input_validation_ip_0_0_0_0() {
        assert_eq!(validate_ipv4("0.0.0.0".to_string()), true)
    }
    #[test]
    fn input_validation_ip_255_255_255_255() {
        assert_eq!(validate_ipv4("255.255.255.255".to_string()), true)
    }
    #[test]
    fn input_validation_ip_192_168_2_1() {
        assert_eq!(validate_ipv4("192.168.2.1".to_string()), true)
    }
    #[test]
    fn input_validation_ip_10_0_0_1() {
        assert_eq!(validate_ipv4("10.0.0.1".to_string()), true)
    }
    #[test]
    fn input_validation_ip_10_0_0_1_dot() {
        assert_eq!(validate_ipv4("10.0.0.1.".to_string()), false)
    }
    #[test]
    fn input_validation_ip_10_0_0_() {
        assert_eq!(validate_ipv4("10.0.0.".to_string()), false)
    }
    #[test]
    fn input_validation_ip_1() {
        assert_eq!(validate_ipv4("1".to_string()), false)
    }
    #[test]
    fn input_validation_ip_qwe() {
        assert_eq!(validate_ipv4("qwe".to_string()), false)
    }
    #[test]
    fn input_validation_ip_196_168_2_256() {
        assert_eq!(validate_ipv4("192.168.2.256".to_string()), false)
    }
    #[test]
    fn input_validation_ip_999_999_999_999() {
        assert_eq!(validate_ipv4("999.999.999.999".to_string()), false)
    }
    #[test]
    fn input_validation_ip_256_256_256_256() {
        assert_eq!(validate_ipv4("256.256.256.256".to_string()), false)
    }
    // ============== Port Validation ===============================
    #[test]
    fn input_validation_port_letters() {
        assert_eq!(validate_port("abc".to_string()), false)
    }
    #[test]
    fn input_validation_port_negative_1() {
        assert_eq!(validate_port("-1".to_string()), false)
    }
    #[test]
    fn input_validation_port_0() {
        assert_eq!(validate_port("0".to_string()), false)
    }
    #[test]
    fn input_validation_port_1024() {
        assert_eq!(validate_port("1024".to_string()), true)
    }
    #[test]
    fn input_validation_port_23000() {
        assert_eq!(validate_port("23000".to_string()), true)
    }
    #[test]
    fn input_validation_port_65534() {
        assert_eq!(validate_port("65534".to_string()), true)
    }
    #[test]
    fn input_validation_port_65535() {
        assert_eq!(validate_port("65535".to_string()), false)
    }
}
