use std::fmt;
use std::cmp;
use std::error::Error as StdError;

use std::collections::HashSet;

const DEFAULT_ALPHABET: &'static str =  "abdegjklmnopqrvwxyzABDEGJKLMNOPQRVWXYZ1234567890";
const DEFAULT_SEPARATORS: &'static str = "cfhistuCFHISTU";
const SEPARATOR_DIV: f32 = 3.5;
const GUARD_DIV: f32 = 12.0;
const MIN_ALPHABET_LENGTH: usize = 16;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuildError {
  ShortAlphabet,
  SpaceInAlphabet,
}

impl fmt::Display for BuildError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.description())
  }
}

impl StdError for BuildError {
  fn description(&self) -> &str {
    match *self {
      BuildError::ShortAlphabet => "Invalid Alphabet Length",
      BuildError::SpaceInAlphabet => "Space character found in alphabet",
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DecodeError {
  InternalGuardChars,
  NonAlphabetChars,
}

impl fmt::Display for DecodeError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.description())
  }
}

impl StdError for DecodeError {
  fn description(&self) -> &str {
    match *self {
      DecodeError::InternalGuardChars => "Hash contains more than 2 guard characters",
      DecodeError::NonAlphabetChars => "Hash contains a character not found in the alphabet",
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashIds {
  salt: Box<[char]>,
  alphabet: Box<[char]>,
  separators: Box<[char]>,
  min_hash_length: usize,
  guards: Box<[char]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HashIdsBuilder {
  salt: Box<[char]>,
  min_length: usize,
  alphabet: Option<Vec<char>>,
}

impl HashIdsBuilder {
  pub fn new() -> Self {
    HashIdsBuilder::default()
  }

  pub fn salt(mut self, salt: &str) -> Self {
    self.salt = salt.chars().collect::<Vec<char>>().into_boxed_slice();
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

  pub fn build(self) -> Result<HashIds, BuildError> {
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
          return Err(BuildError::SpaceInAlphabet);
        }
        for ch in DEFAULT_SEPARATORS.chars() {
          if alphabet_set.contains(&ch) {
            separators.push(ch);
          }
        }
        if alphabet.len() + separators.len() < MIN_ALPHABET_LENGTH {
          return Err(BuildError::ShortAlphabet);
        }
        (alphabet, separators)
      }
      None => {
        (DEFAULT_ALPHABET.chars().collect(), DEFAULT_SEPARATORS.chars().collect())
      }
    };
    
    shuffle(&mut separators, &salt);
    if custom_alphabet {
      let expected_sep_len = (alphabet.len() as f32 / SEPARATOR_DIV).ceil() as usize;
      if separators.len() < expected_sep_len {
        let diff = expected_sep_len - separators.len();
        // Steal the first `diff` chars from alphabet, and add to separators
        separators.extend(alphabet.drain(..diff));
      }
    }
    shuffle(&mut alphabet, &salt);
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

  pub fn with_salt(salt: &str) -> Self {
    // Can only fail with custom alphabet
    HashIdsBuilder::new().salt(salt).build().unwrap()
  }

  pub fn decode(&self, hash: &str) -> Result<Vec<u64>, DecodeError> {
    let guards = &self.guards[..];
    let hash_start = hash
        .char_indices()
        .find(|&(_, ch)| guards.contains(&ch))
        .map(|(i, ch)| i + ch.len_utf8())
        .unwrap_or(0);
    let hash_end = if hash_start == 0 {
      hash.len()
    } else {
      hash[hash_start..]
          .char_indices()
          .rev()
          .find(|&(_, ch)| guards.contains(&ch))
          .map(|(i, _ch)| hash_start + i)
          .unwrap_or_else(|| hash.len())
    };
    let hash = &hash[hash_start..hash_end];

    if hash.is_empty() {
      return Ok(vec![]);
    }
    if hash.chars().any(|ch| self.guards.contains(&ch)) {
      // If any guard characters are left, hash is invalid
      return Err(DecodeError::InternalGuardChars);
    }

    let mut result = Vec::new();

    let mut hash_chars = hash.chars();
    // Safe because hash_chars is not empty
    let lottery = hash_chars.next().unwrap();
    let hash = hash_chars.as_str();
    let mut alphabet = self.alphabet.clone();
    let mut buffer = Vec::with_capacity(alphabet.len());
    buffer.push(lottery);
    let needed_salt = cmp::min(self.salt.len(), alphabet.len() - 1);
    buffer.extend_from_slice(&self.salt[..needed_salt]);
    let const_buffer_len = buffer.len();
    for sub_hash in hash.split(|ch| self.separators.contains(&ch)) {
      buffer.truncate(const_buffer_len);
      if buffer.len() < alphabet.len() {
        let extra_needed = alphabet.len() - buffer.len();
        buffer.extend_from_slice(&alphabet[..extra_needed]);
      }
      shuffle(&mut alphabet, &buffer);

      if let Some(number) = unhash(sub_hash, &alphabet) {
        result.push(number);
      } else {
        return Err(DecodeError::NonAlphabetChars);
      }
    }

    Ok(result)
  }

  pub fn encode(&self, numbers: &[u64]) -> String {
    if numbers.len() == 0 {
      return String::new();
    }

    let mut number_hash_int: u64 = 0;
    for (i, &number) in numbers.iter().enumerate() {
      number_hash_int = number_hash_int.wrapping_add(number % (i as u64 + 100));
    }

    let lottery_idx = (number_hash_int % self.alphabet.len() as u64) as usize;
    let lottery = self.alphabet[lottery_idx];
    let mut result: Vec<char> = Vec::with_capacity(self.min_hash_length);
    result.push(lottery);

    let mut alphabet = self.alphabet.clone();
    let mut buffer = Vec::with_capacity(alphabet.len());
    buffer.push(lottery);
    let needed_salt = cmp::min(self.salt.len(), alphabet.len() - 1);
    buffer.extend_from_slice(&self.salt[..needed_salt]);
    let const_buffer_len = buffer.len();
    for (i, &number) in numbers.iter().enumerate() {
      buffer.truncate(const_buffer_len);
      // Don't bother adding anything from alphabet if buffer is long enough already
      if const_buffer_len < alphabet.len() {
          let extra_needed = alphabet.len() - buffer.len();
          buffer.extend_from_slice(&alphabet[..extra_needed]);
      }
      shuffle(&mut alphabet, &buffer);
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
      buffer.extend_from_slice(&alphabet);
      shuffle(&mut alphabet, &buffer);

      let excess = (result.len() + alphabet.len()).saturating_sub(self.min_hash_length);
      let start_pos = excess / 2;
      result.splice(0..0, alphabet[half_len + start_pos..].iter().cloned());
      result.extend_from_slice(&alphabet[..half_len - (excess - start_pos)]);
    }

    result.into_iter().collect()
  }
}

fn shuffle(alphabet: &mut [char], salt: &[char]) {
  if salt.is_empty() {
    return;
  }

  let len = alphabet.len();
  let salt_len = salt.len();

  let mut v: usize = 0;
  let mut p: usize = 0;

  for i in (1..len).rev() {
    v %= salt_len;
    // safe because of the above modulus
    let t = salt[v] as usize;
    p = p.wrapping_add(t);
    let j = (t.wrapping_add(v).wrapping_add(p)) % i;

    alphabet.swap(i, j);

    v += 1;
  }
}

fn unhash(input: &str, alphabet: &[char]) -> Option<u64> {
  let input_len = input.chars().count();
  let mut number: u64 = 0;
  for (i, ch) in input.chars().enumerate() {
    if let Some(pos) = alphabet.iter().position(|&x| x == ch) {
      number += pos as u64 * (alphabet.len() as u64).pow((input_len - i - 1) as u32);
    } else {
      return None;
    }
  }
  Some(number)
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

