extern crate hashids;
#[macro_use]
extern crate quickcheck;

use quickcheck::TestResult;

use hashids::{HashIds, HashIdsBuilder};
use std::str;
use std::process::Command;

#[test]
fn it_works_1() {
  let ids = HashIds::with_salt(String::from("this is my salt"));

  let numbers: Vec<u64> = vec![12345];
  let encode = ids.encode(&numbers);
  println!("{}", encode);
  let longs = ids.decode(&encode).unwrap();

  assert_eq!(numbers, longs);
}

fn run_javascript(salt: &str, alphabet: &str, min_len: usize, nums: &[u64]) -> Result<String, String> {
  let mut command = Command::new("node");

  command
    .arg("-e")
    .arg(include_str!("javascript_hashids.js"))
    .arg("--")
    .args(nums.iter().map(|i| i.to_string()));

  command
    .arg("-m")
    .arg(&min_len.to_string());
  if ! salt.is_empty() {
    command
      .arg("-s")
      .arg(salt);
  }
  if ! alphabet.is_empty() {
    command
      .arg("-a")
      .arg(alphabet);
  }

  let mut output = command
    .output()
    .unwrap();
  output.stdout.pop();
  output.stderr.pop();
  if output.status.success() {
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
  } else {
    Err(String::from_utf8_lossy(&output.stderr).into_owned())
  }
}

#[test]
fn default_equal_explicit() {
  let ids_def = HashIds::new();
  let ids_exp = HashIdsBuilder::new()
    .salt(String::new())
    .alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
    .build()
    .unwrap();
  let data = [1, 2, 3, 4, 999];
  assert_eq!(ids_def.encode(&data), run_javascript("", "", 0, &data).unwrap());
  assert_eq!(ids_def.encode(&data), ids_exp.encode(&data));
}

#[test]
fn bad() {
  // let alphabet = [1, 72, 2, 4, 5, 6, 7, 8, 70, 9, 10, 11, 12, 13, 14, 3];
  // \x01\x48\x02\x04\x05\x06\x07\x08\x46\x09\x0A\x0B\x0C\x0D\x0E\x03
  let alphabet = [1, 72, 2, 4, 5, 6, 7, 8, 70, 9, 10, 11, 12, 13, 14, 3];
  let alphabet = str::from_utf8(&alphabet).unwrap();
  let ids = HashIdsBuilder::new()
    .alphabet(alphabet)
    .build()
    .unwrap();
  let data = [0, 0];
  assert_eq!(ids.encode(&data), run_javascript("", alphabet, 0, &data).unwrap());
}


quickcheck! {
  fn equals_javascript(salt: Vec<u8>, alphabet: Vec<u8>, min_len: usize, nums: Vec<u64>) -> TestResult {
    if nums.is_empty() {
      return TestResult::discard();
    }
    // make alphabet ascii
    let mut alphabet = alphabet;
    for ch in alphabet.iter_mut() {
      *ch = *ch & 0x7F;
    }
    let alphabet = String::from_utf8(alphabet).unwrap();
    // make salt ascii
    let mut salt = salt;
    for ch in salt.iter_mut() {
      *ch = *ch & 0x7F;
    }
    let salt = String::from_utf8(salt).unwrap();
    if salt.contains('\0') || alphabet.contains('\0') {
      return TestResult::discard();
    }
    let js_result = run_javascript(&salt, &alphabet, min_len, &nums);
    let mut builder = HashIdsBuilder::new().min_length(min_len);
    if ! salt.is_empty() {
      builder = builder.salt(salt);
    }
    if ! alphabet.is_empty() {
      builder = builder.alphabet(&alphabet)
    }

    let ids = builder.build();

    match ids {
      Ok(ids) => {
        assert_eq!(ids.encode(&nums), js_result.unwrap());
      }
      Err(_) => {
        assert!(js_result.is_err(), "{:?}", js_result);
        return TestResult::discard();
      }
    }

    TestResult::passed()
  }
}

/*
#[test]
fn it_works_2() {
  let ids = HashIds::new(String::from("this is my salt")).unwrap();

  let numbers: Vec<u64> = vec![12345];
  let encode = ids.encode(&numbers);

  let ids2 = HashIds::new(String::from("this is my pepper")).unwrap();

  assert!(ids2.decode(&encode).is_none());
}

#[test]
fn it_works_3() {
  let ids = HashIds::new(String::from("this is my salt")).unwrap();

  let numbers: Vec<u64> = vec![683, 94108, 123, 5];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "aBMswoO2UB3Sj");
}

#[test]
fn it_works_4() {
  let ids = HashIds::with_min_length(String::from("this is my salt"), 8).unwrap();

  let numbers: Vec<u64> = vec![1];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "gB0NV05e");
}

#[test]
fn it_works_5() {
  let ids = HashIds::with_min_length_and_alphabet(String::from("this is my salt"), 0, "0123456789abcdef").unwrap();

  let numbers: Vec<u64> = vec![1234567];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "b332db5");
}

#[test]
fn it_works_6() {
  let ids = HashIds::new(String::from("this is my salt")).unwrap();

  let numbers: Vec<u64> = vec![5, 5, 5, 5];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "1Wc8cwcE");
}

#[test]
fn it_works_7() {
  let ids = HashIds::new(String::from("this is my salt")).unwrap();

  let numbers: Vec<u64> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "kRHnurhptKcjIDTWC3sx");
}

#[test]
fn it_works_8() {
  let ids = HashIds::new(String::from("this is my salt")).unwrap();

  let numbers_1: Vec<u64> = vec![1];
  let encode_1 = ids.encode(&numbers_1);
  let numbers_2: Vec<u64> = vec![2];
  let encode_2 = ids.encode(&numbers_2);
  let numbers_3: Vec<u64> = vec![3];
  let encode_3 = ids.encode(&numbers_3);
  let numbers_4: Vec<u64> = vec![4];
  let encode_4 = ids.encode(&numbers_4);
  let numbers_5: Vec<u64> = vec![5];
  let encode_5 = ids.encode(&numbers_5);

  assert_eq!(encode_1, "NV");
  assert_eq!(encode_2, "6m");
  assert_eq!(encode_3, "yD");
  assert_eq!(encode_4, "2l");
  assert_eq!(encode_5, "rD");
}

*/