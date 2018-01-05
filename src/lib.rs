use std::fmt;
use std::error::Error as StdError;

use std::collections::HashSet;

const DEFAULT_ALPHABET: &'static str =  "abdegjklmnopqrvwxyzABDEGJKLMNOPQRVWXYZ1234567890";
const DEFAULT_SEPARATORS: &'static str = "cfhistuCFHISTU";
const SEPARATOR_DIV: f32 = 3.5;
const GUARD_DIV: f32 = 12.0;
const MIN_ALPHABET_LENGTH: usize = 16;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
  ShortAlphabet,
  SpaceInAlphabet,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.description())
  }
}

impl StdError for Error {
  fn description(&self) -> &str {
    match *self {
      Error::ShortAlphabet => "Invalid Alphabet Length",
      Error::SpaceInAlphabet => "Space character found in alphabet",
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashIds {
  salt: Box<str>,
  alphabet: Box<[char]>,
  separators: Box<[char]>,
  min_hash_length: usize,
  guards: Box<[char]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HashIdsBuilder {
  salt: Box<str>,
  min_length: usize,
  alphabet: Option<Vec<char>>,
}

impl HashIdsBuilder {
  pub fn new() -> Self {
    HashIdsBuilder::default()
  }

  pub fn salt(mut self, salt: String) -> Self {
    self.salt = salt.into_boxed_str();
    self
  }

  pub fn min_length(mut self, min_length: usize) -> Self {
    self.min_length = min_length;
    self
  }

  pub fn alphabet(mut self, alphabet: &str) -> Self {
    self.alphabet = Some(alphabet.chars().collect());
    self
  }

  pub fn build(self) -> Result<HashIds, Error> {
    let HashIdsBuilder {
      salt,
      min_length,
      alphabet,
    } = self;
    let custom_alphabet = alphabet.is_some();
    let (mut alphabet, mut separators) = match alphabet {
      Some(mut alphabet) => {
        let mut alphabet_set = HashSet::with_capacity(alphabet.len());
        let mut separators = Vec::with_capacity(DEFAULT_SEPARATORS.len());
        let mut contains_space = false;
        alphabet.retain(|&ch| {
          if ch == ' ' {
            contains_space = true;
          }
          if alphabet_set.insert(ch) {
            if DEFAULT_SEPARATORS.contains(ch) {
              false
            } else {
              true
            }
          } else {
            false
          }
        });
        if contains_space {
          return Err(Error::SpaceInAlphabet);
        }
        for ch in DEFAULT_SEPARATORS.chars() {
          if alphabet_set.contains(&ch) {
            separators.push(ch);
          }
        }
        if alphabet.len() + separators.len() < MIN_ALPHABET_LENGTH {
          return Err(Error::ShortAlphabet);
        }
        (alphabet, separators)
      }
      None => {
        (DEFAULT_ALPHABET.chars().collect(), DEFAULT_SEPARATORS.chars().collect())
      }
    };
    
    shuffle(&mut separators, salt.as_bytes());
    if custom_alphabet {
      let expected_sep_len = (alphabet.len() as f32 / SEPARATOR_DIV).ceil() as usize;
      if separators.len() < expected_sep_len {
        let diff = expected_sep_len - separators.len();
        // Steal the first `diff` chars from alphabet, and add to separators
        separators.extend(alphabet.drain(..diff));
      }
    }
    shuffle(&mut alphabet, salt.as_bytes());
    let guard_count = (alphabet.len() as f32 / GUARD_DIV).ceil() as usize;
    let guards: Vec<char>;
    {
      let guard_source = if alphabet.len() < 3 {
        &mut separators
      } else {
        &mut alphabet
      };
      guards = guard_source.drain(..guard_count).collect();
    }
    Ok(HashIds {
      salt: salt,
      alphabet: alphabet.into_boxed_slice(),
      separators: separators.into_boxed_slice(),
      guards: guards.into_boxed_slice(),
      min_hash_length: min_length,
    })
  }
}

impl HashIds {
  pub fn new() -> Self {
    // Can only fail with custom alphabet
    HashIdsBuilder::new().build().unwrap()
  }

  pub fn with_salt(salt: String) -> Self {
    // Can only fail with custom alphabet
    HashIdsBuilder::new().salt(salt).build().unwrap()
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
          result.push_str(&buffer[1..]);
          buffer.clear();
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
      shuffle(&mut alphabet, buffer.as_bytes());

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

  pub fn encode(&self, numbers: &[u64]) -> String {
    if numbers.len() == 0 {
      panic!("Unable to encode an empty slice of numbers");
    }

    self._encode(numbers)
  }

  fn _encode(&self, numbers: &[u64]) -> String {
    let mut number_hash_int: u64 = 0;
    for (i, &number) in numbers.iter().enumerate() {
      number_hash_int = number_hash_int.wrapping_add(number % (i as u64 + 100));
    }

    let lottery_idx = (number_hash_int % self.alphabet.len() as u64) as usize;
    let lottery = self.alphabet[lottery_idx];
    let mut result: Vec<char> = Vec::with_capacity(self.min_hash_length);
    result.push(lottery);

    let mut alphabet = self.alphabet.clone();
    let mut buffer = String::with_capacity(alphabet.len());
    buffer.push(lottery);
    buffer.push_str(&self.salt);
    let const_buffer_len = buffer.len();
    for (i, &number) in numbers.iter().enumerate() {
      buffer.truncate(const_buffer_len);
      // Don't bother adding anything from alphabet if buffer is long enough already
      if const_buffer_len < alphabet.len() {
          let extra_needed = alphabet.len() - buffer.len();
          buffer.extend(alphabet[..extra_needed].iter().cloned());
      }
      shuffle(&mut alphabet, buffer.as_bytes());
      let last_start = result.len();
      add_num_with_alphabet(&mut result, number, &alphabet);

      if (i + 1) < numbers.len() {
        let sep_idx = number % (result[last_start] as u64 + i as u64);
        let sep_idx =  (sep_idx % self.separators.len() as u64) as usize;
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
      buffer.extend(alphabet.iter().cloned());
      shuffle(&mut alphabet, buffer.as_bytes());

      let excess = (result.len() + alphabet.len()).saturating_sub(self.min_hash_length);
      let start_pos = excess / 2;
      result.splice(0..0, alphabet[half_len + start_pos..].iter().cloned());
      result.extend_from_slice(&alphabet[..half_len - (excess - start_pos)]);
    }

    result.into_iter().collect()
  }
}

fn shuffle(alphabet: &mut [char], salt: &[u8]) {
  if salt.len() <= 0 {
    return;
  }

  let len = alphabet.len();

  let mut v: usize = 0;
  let mut p: usize = 0;

  for i in (1..len).rev() {
    v %= salt.len();
    let t = salt[v] as usize;
    p += t;
    let j = (t.wrapping_add(v).wrapping_add(p)) % i;

    alphabet.swap(i, j);

    v += 1;
  }
}

fn add_num_with_alphabet(hash: &mut Vec<char>, mut input: u64, alphabet: &[char]) {
  let new_start = hash.len();
  let len = alphabet.len() as u64;

  loop {
    let idx = (input % len) as usize;
    let ch = alphabet[idx];
    hash.push(ch);
    input /= len;

    if input == 0 {
      break;
    }
  }

  hash[new_start..].reverse();
}

