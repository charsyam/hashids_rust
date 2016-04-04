#![crate_name = "hashids"]

use std::fmt;
use std::error::Error;

const DEFAULT_ALPHABET: &'static str =  "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
const DEFAULT_SEPARATORS: &'static str = "cfhistuCFHISTU";
const SEPARATOR_DIV: f32 = 3.5;
const GUARD_DIV: f32 = 12.0;
const MIN_ALPHABET_LENGTH: usize = 16;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HashIdsError { InvalidAlphabetLength }

impl fmt::Display for HashIdsError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.description())
  }
}

impl Error for HashIdsError {
  fn description(&self) -> &str {
    "Invalid Alphabet Length"
  }
}

pub struct HashIds {
  salt: Box<str>,
  alphabet: Box<[char]>,
  separators: Box<[char]>,
  min_hash_length: usize,
  guards: Box<[char]>,
}

impl HashIds {
  pub fn with_min_length_and_alphabet(salt: String, min_length: usize, alphabet: &str) -> Result<HashIds, HashIdsError>
  {
    let mut alphabet = {
      let mut unique_alphabet = Vec::with_capacity(alphabet.len());
      for ch in alphabet.chars() {
        if !unique_alphabet.contains(&ch) {
          unique_alphabet.push(ch);
        }
      }
      unique_alphabet
    };

    if alphabet.len() < MIN_ALPHABET_LENGTH {
      return Err(HashIdsError::InvalidAlphabetLength);
    }

    let mut separators: Vec<char> = DEFAULT_SEPARATORS.chars().collect();
    for i in (0..separators.len()).rev() {
      match alphabet.iter().position(|&ch| ch == separators[i]) {
        Some(idx) => {
          alphabet.remove(idx);
        },
        None => {
          separators.remove(i);
        }
      }
    }

    HashIds::shuffle(&mut separators, &salt);


    if separators.is_empty() || (alphabet.len() as f32 / separators.len() as f32) > SEPARATOR_DIV {
      let mut seps_len = ((alphabet.len() as f32) / SEPARATOR_DIV).ceil() as usize;
      if seps_len == 1 {
        seps_len += 1;
      }

      if seps_len > separators.len() {
        let diff = seps_len - separators.len();

        separators.extend_from_slice(&alphabet[..diff]);
        alphabet.drain(..diff);
      } else {
        separators.truncate(seps_len);
      }
    }

    HashIds::shuffle(&mut alphabet, &salt);
    let guard_count = (alphabet.len() as f32 / GUARD_DIV).ceil() as usize;

    let guards;

    if alphabet.len() < 3 {
      guards = separators[..guard_count].to_vec();
      separators.drain(..guard_count);
    } else {
      guards = alphabet[..guard_count].to_vec();
      alphabet.drain(..guard_count);
    }

    Ok(HashIds {
      salt: salt.into_boxed_str(),
      min_hash_length: min_length,
      guards: guards.into_boxed_slice(),
      separators: separators.into_boxed_slice(),
      alphabet: alphabet.into_boxed_slice(),
    })
  }

  pub fn with_min_length(salt: String, min_hash_length: usize) -> Result<HashIds, HashIdsError> {
    HashIds::with_min_length_and_alphabet(salt, min_hash_length, DEFAULT_ALPHABET)
  }

  pub fn new(salt: String) -> Result<HashIds, HashIdsError> {
    HashIds::with_min_length(salt, 0)
  }

  pub fn encode_hex(&self, hex: &str) -> Option<String> {
    let mut numbers: Vec<u64> = Vec::with_capacity(hex.len() / 12);
    for chunk in hex.as_bytes().chunks(12) {
      let mut number = 1;
      for &ch in chunk {
        let digit = match ch {
          b'0'...b'9' => ch - b'0',
          b'a'...b'f' => ch - b'a' + 10,
          b'A'...b'F' => ch - b'A' + 10,
          _ => return None,
        } as u64;
        number <<= 4;
        number |= digit;
      }
      numbers.push(number);
    }
    Some(self.encode(&numbers))
  }

  pub fn decode_hex(&self, hash: &str) -> Option<String> {
    use std::fmt::Write;
    match self.decode(hash) {
      Some(numbers) => {
        let mut result = String::new();
        let mut buffer = String::new();
        for number in numbers {
          write!(buffer, "{:x}", number).unwrap();
          result.push_str(&buffer[1..])
        }
        Some(result)
      },
      None => None,
    }
  }

  pub fn decode(&self, hash: &str) -> Option<Vec<u64>> {
    let mut hash_chars: Vec<char> = hash.chars().collect();
    if let Some(end_guard) = hash_chars.iter().rposition(|ch| self.guards.contains(ch)) {
      hash_chars.truncate(end_guard);
    }
    if let Some(start_guard) = hash_chars.iter().position(|ch| self.guards.contains(ch)) {
      hash_chars.drain(..start_guard);
    }
    if hash_chars.iter().any(|ch| self.guards.contains(ch)) {
      // If any guard characters are left, hash is invalid
      return None;
    }
    if hash_chars.is_empty() {
      return None;
    }

    let num_results = hash_chars.iter().filter(|ch| self.separators.contains(ch)).count() + 1;
    let mut result = Vec::with_capacity(num_results);

    let lottery = hash_chars.remove(0);
    let mut alphabet = self.alphabet.clone();
    let mut buffer = String::with_capacity(alphabet.len());
    buffer.push(lottery);
    buffer.push_str(&self.salt);
    if buffer.len() > alphabet.len() {
      buffer.truncate(alphabet.len());
    }
    let const_buffer_len = buffer.len();
    for sub_hash in hash_chars.split(|ch| self.separators.contains(ch)) {
      buffer.truncate(const_buffer_len);
      if buffer.len() < alphabet.len() {
        let extra_needed = alphabet.len() - buffer.len();
        buffer.extend(alphabet[..extra_needed].iter());
      }
      HashIds::shuffle(&mut alphabet, &buffer);

      if let Some(number) = HashIds::unhash(sub_hash, &alphabet) {
        result.push(number);
      } else {
        return None;
      }
    }

    if cfg!(debug_assertions) {
      let check_hash = self._encode(&result);
      if check_hash != hash {
        return None;
      }
    }

    Some(result)
  }

  fn unhash(input: &[char], alphabet: &[char]) -> Option<u64> {
    let mut number: u64 = 0;
    for (i, ch) in input.iter().enumerate() {
      if let Some(pos) = alphabet.iter().position(|x| x == ch) {
        number += pos as u64 * (alphabet.len() as u64).pow(input.len() as u32 - i as u32 - 1);
      } else {
        return None;
      }
    }
    Some(number)
  }

  fn hash(mut input: u64, alphabet: &[char]) -> Vec<char> {
    let len = alphabet.len() as u64;
    let mut result = Vec::new();

    loop {
      let idx = (input % len) as usize;
      let ch = alphabet[idx];
      result.push(ch);
      input /= len;

      if input == 0 {
        break;
      }
    }

    result.reverse();
    result
  }

  pub fn encode(&self, numbers: &[u64]) -> String {
    if numbers.len() == 0 {
      panic!("Unable to encode an empty slice of numbers");
    }

    self._encode(numbers)
  }

  fn _encode(&self, numbers: &[u64]) -> String {
    let mut number_hash_int = 0;
    for (i, &number) in numbers.iter().enumerate() {
      number_hash_int += number % (i as u64 + 100);
    }

    let lottery_idx = (number_hash_int % self.alphabet.len() as u64) as usize;
    let lottery = self.alphabet[lottery_idx];
    let mut result: Vec<char> = Vec::with_capacity(self.min_hash_length);
    result.push(lottery);

    let mut alphabet = self.alphabet.clone();
    let mut buffer = String::with_capacity(alphabet.len());
    buffer.push(lottery);
    buffer.push_str(&self.salt);
    if buffer.len() > alphabet.len() {
      buffer.truncate(alphabet.len());
    }
    let const_buffer_len = buffer.len();
    for (i, &number) in numbers.iter().enumerate() {
      buffer.truncate(const_buffer_len);
      // Don't bother adding anything from alphabet if buffer is long enough already
      if buffer.len() < alphabet.len() {
          let extra_needed = alphabet.len() - buffer.len();
          buffer.extend(alphabet[..extra_needed].iter());
      }
      HashIds::shuffle(&mut alphabet, &buffer);
      let last = HashIds::hash(number, &alphabet);

      result.extend_from_slice(&last);

      if (i + 1) < numbers.len() {
        let mut sep_idx = (number % (last[0] as u64 + i as u64)) as usize;
        sep_idx %= self.separators.len();
        result.push(self.separators[sep_idx]);
      }
    }

    if result.len() < self.min_hash_length {
      let guard_idx = ((number_hash_int + result[0] as u64) % self.guards.len() as u64) as usize;
      let guard = self.guards[guard_idx];
      result.insert(0, guard);

      if result.len() < self.min_hash_length {
        let guard_idx = ((number_hash_int + result[2] as u64) % self.guards.len() as u64) as usize;
        let guard = self.guards[guard_idx];
        result.push(guard);
      }
    }

    let half_len = alphabet.len() / 2;
    while result.len() < self.min_hash_length {
      buffer.clear();
      buffer.extend(alphabet.iter());
      HashIds::shuffle(&mut alphabet, &buffer);

      result = alphabet[half_len..].iter().cloned().chain(result.into_iter()).chain(alphabet[..half_len].iter().cloned()).collect();

      let excess = result.len() - self.min_hash_length;
      if excess > 0 {
        let start_pos = excess / 2;
        result.truncate(start_pos + self.min_hash_length);
        result.drain(..start_pos);
      }
    }

    result.into_iter().collect()
  }

  fn shuffle(alphabet: &mut [char], salt: &str) {
    let salt = salt.as_bytes();
    if salt.len() <= 0 {
      return;
    }

    let len = alphabet.len();

    let mut i: usize = len-1;
    let mut v: usize = 0;
    let mut p: usize = 0;

    while i > 0 {
      v %= salt.len();
      let t = salt[v] as usize;
      p += t;
      let j = (t + v + p) % i;

      alphabet.swap(i,j);

      i=i-1;
      v=v+1; 
    }
  }
}
