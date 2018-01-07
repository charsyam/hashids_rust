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
  let ids = HashIdsBuilder::new()
    .min_length(3)
    .build()
    .unwrap();
  let data = [0];
  let encoded = ids.encode(&data);
  assert_eq!(encoded, run_javascript("", "", 3, &data).unwrap());
  let decoded = ids.decode(&encoded).unwrap();
  assert_eq!(decoded, data);
}


quickcheck! {
  fn equals_javascript(salt: Vec<u8>, alphabet: Vec<u8>, min_len: usize, nums: Vec<u64>) -> TestResult {
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
        let encoded = ids.encode(&nums);
        assert_eq!(encoded, js_result.unwrap());
        let decoded = ids.decode(&encoded).unwrap();
        assert_eq!(decoded, nums);
      }
      Err(_) => {
        assert!(js_result.is_err(), "{:?}", js_result);
        return TestResult::discard();
      }
    }

    TestResult::passed()
  }
}
