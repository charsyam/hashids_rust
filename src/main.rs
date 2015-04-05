extern crate regex;

static DEFAULT_ALPHABET: &'static str =  "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";

static DEFAULT_SEPARATORS: &'static str = "cfhistuCFHISTU";
static SEP_DIV: f32 = 3.5;
static GUARD_DIV: u32 = 12;
static MIN_ALPHABET_LENGTH: usize = 16;


struct HashIds {
  salt: String,
  alphabet: String,
  separators: String,
  minHashLength: usize,
  guards: String 
}

impl HashIds {

  pub fn new(salt: String, minHashLength: usize, alphabet: String) -> HashIds {
    use std::num::Float;

    let minLength = HashIds::getMinHashLength(minHashLength);
    let uniqueAlphabet = HashIds::getUniqueAlphabet(alphabet);

    if uniqueAlphabet.len() < ::MIN_ALPHABET_LENGTH {
      //Error
    }

    let (t_separators, mut t_alphabet) = HashIds::getSeps(uniqueAlphabet);
    let mut shuffledSeparators = HashIds::hashids_shuffle(t_separators.clone(), salt.clone());

    if HashIds::needManipulate(shuffledSeparators.len(), t_alphabet.len()) == true {
      let mut seps_len =  ((t_alphabet.len() as f32) / ::SEP_DIV) as usize;
      if seps_len == 1 {
        seps_len += 1;
      }

      if seps_len > shuffledSeparators.len() {
        let diff = seps_len - shuffledSeparators.len();
        shuffledSeparators.push_str(t_alphabet.slice(0,diff));
        t_alphabet = t_alphabet.slice_from(diff).to_string();
      } else {
        shuffledSeparators = shuffledSeparators.slice(0, seps_len).to_string();
      }
    }

    let mut shuffledAlphabet = HashIds::hashids_shuffle(t_alphabet.clone(), salt.clone());
    let guardCount = (shuffledAlphabet.len() as f32 / ::GUARD_DIV as f32).ceil() as usize;

    let mut t_guards;

    if guardCount < 3 {
      t_guards = shuffledSeparators.slice(0, guardCount).to_string();
      shuffledSeparators = shuffledSeparators.slice_from(guardCount).to_string(); 
    } else {
      t_guards = shuffledAlphabet.slice(0, guardCount).to_string();
      shuffledAlphabet = shuffledAlphabet.slice_from(guardCount).to_string(); 
    }
    
    HashIds {
      salt: salt.to_string(),
      minHashLength: minLength,
      guards: t_guards,
      separators: shuffledSeparators,
      alphabet: shuffledAlphabet
    }
  }

  pub fn newWithSaltAndMinLength(salt: String, minHashLength: usize) -> HashIds {
    HashIds::new(salt, minHashLength, DEFAULT_ALPHABET.to_string())
  }

  pub fn newWithSalt(salt: String) -> HashIds {
    HashIds::newWithSaltAndMinLength(salt, 0)
  }


  fn needManipulate(slen: usize, alen: usize) -> bool {
    if slen <= 0 && (((alen/slen) as f32)> ::SEP_DIV) {
      return true;
    }

    false
  }

  fn getMinHashLength(length: usize) -> usize {
    if length > 0 {
      return length;
    } else {
      return 0;
    }
  }

  fn getSeps(alphabet: String) -> (String, String) {
    HashIds::getNonDuplicatedString(::DEFAULT_SEPARATORS.to_string(), alphabet)
  }

  fn hashids_shuffle(alphabet: String, salt: String) -> String {
    if salt.len() <= 0 {
      return alphabet;
    }

    let salt_len = salt.len();
    let arr = salt.as_bytes();
    let len = alphabet.len();
    let mut bytes = alphabet.into_bytes();
    let mut shuffle = bytes.as_mut_slice();

    let mut i: usize = len-1;
    let mut j: usize = 0;
    let mut v: usize = 0;
    let mut p: usize = 0;
    let mut temp1: u8;
    let mut temp2: u8;
    let mut t: usize = 0;

    while i > 0 {
      v %= salt_len;
      t = arr[v] as usize;
      p += t;
      j = (t + v + p) % i;

      shuffle.swap(i,j);

      i=i-1;
      v=v+1; 
    }

    let mut shuffledAlphabet = String::with_capacity(len);
    for i in (0..len) {
      shuffledAlphabet.push(shuffle[i] as char);
    }

    shuffledAlphabet
  }

  fn getNonDuplicatedString(separators: String, alphabet: String) -> (String, String) {
    use std::collections::HashMap;
    let mut checkSeparatorMap = HashMap::new();
    let mut checkAlphabetMap = HashMap::new();

    let mut modifiedSeparators = String::new();
    let mut modifiedAlphabet = String::new();
    
    for c in separators.chars() {
      checkSeparatorMap.insert(c, 1);
    }

    for c in alphabet.chars() {
      checkAlphabetMap.insert(c, 1);
    }

    for c in separators.chars() {
      if checkAlphabetMap.contains_key(&c) {
        modifiedSeparators.push(c);
      }
    }
    
    for c in alphabet.chars() {
      if !checkSeparatorMap.contains_key(&c) {
        modifiedAlphabet.push(c);
      }
    }
    
    (modifiedSeparators, modifiedAlphabet)
  }

  fn getUniqueAlphabet(alphabet: String) -> String {
    use std::collections::HashMap;

    let mut uniqueAlphabet: String = String::new();
    let mut checkMap = HashMap::new();
    
    for c in alphabet.chars() {
      if !checkMap.contains_key(&c) {
        uniqueAlphabet.push(c);
        checkMap.insert(c, 1);
      }
    }

    uniqueAlphabet
  }

  fn encrypt(&self, numbers: &Vec<i64>) -> String {
    return self.encode(numbers);
  }

  fn decrypt(&self, hash: String) -> Vec<i64> {
    return self.decode(hash);
  }

  fn encode(&self, numbers: &Vec<i64>) -> String {
    let mut ret = "".to_string();
    if numbers.len() == 0 {
      return ret;
    }

    for number in numbers.iter() {
      if *number > 9007199254740992 {
        return ret;
      }
    }

    self._encode(numbers)
  }

  fn decode(&self, hash: String) -> Vec<i64> {  
    let ret : Vec<i64> = Vec::new();
    if hash.len() == 0 {
      return ret;
    }

    self._decode(hash)
  }

  fn _decode(&self, hash: String) -> Vec<i64> {
    use regex::Regex;

    let mut regexp = String::new();
    regexp.push('[');
    regexp.push_str(self.guards.as_slice());
    regexp.push(']');

    let re = Regex::new(regexp.as_slice()).unwrap();
    let t_hash = re.replace_all(hash.as_slice(), " ");

    let mut split: Vec<&str> = t_hash.as_slice().split_str(" ").collect();
    let mut i = 0;

    let len = split.len();
    if len == 3 || len == 2 {
      i = 1;
    }
    let mut hashBreakdown = split[i].to_string();

    let lottery = hashBreakdown.slice(0,1).to_string();

    hashBreakdown = hashBreakdown.slice_from(1).to_string();
    
    let mut regexp2 = String::new();
    regexp2.push('[');
    regexp2.push_str(self.separators.as_slice());
    regexp2.push(']');

    let re2 = Regex::new(regexp2.as_slice()).unwrap();
    hashBreakdown  = re2.replace_all(hashBreakdown.as_slice(), " ");

    split = hashBreakdown.as_slice().split_str(" ").collect();

    let mut alphabet = self.alphabet.clone();

    let mut ret: Vec<i64> = Vec::new();

    for s in split.iter() {
      let subHash = s.to_string();
      let mut buffer = String::new();
      buffer.push_str(lottery.as_slice());
      buffer.push_str(self.salt.as_slice());
      buffer.push_str(alphabet.clone().as_slice());

      let alpha_len = alphabet.len();
      alphabet = HashIds::hashids_shuffle(alphabet, buffer.slice(0, alpha_len).to_string());
      ret.push(HashIds::unhash(subHash, alphabet.clone()));
    }

    ret
  }

  fn indexOf(input :&[u8], v: u8) -> i64 {
    let mut i = 0;
    for s in input.iter() {
      if *s == v {
        return i;
      }

      i += 1;
    }

    return -1;
  }

  fn unhash(input: String, alphabet: String) -> i64 {
    use std::num::Int;

    let mut number: i64 = 0;
    let input_slice = input.as_bytes();
    let alpha_slice = alphabet.as_bytes();
    let len = input.len();
    let alpha_len = alphabet.len() as i64;
    let mut pos : usize = 0;

    let mut i: usize = 0;
    loop {
      if i >= len {
        break;
      }

      let v = input_slice[i] as usize;
      let pos = HashIds::indexOf(alpha_slice, v as u8);
      let pow_size = (len - i - 1) as usize;
      number += ((pos * alpha_len.pow(pow_size)) as i64);
      i += 1;
    }

    number
  }

  fn hash(input: i64, alphabet: String) -> String {
    let mut t_in = input;
    let mut hash = "".to_string();
    let len = alphabet.len() as i64;
    let mut idx = 0;

    loop {
      idx = (t_in % len) as usize;
      let mut t = alphabet.slice(idx, idx+1).to_string();
      t.push_str(hash.as_slice());
      hash = t;
      t_in /= len;

      if t_in <= 0 {
        break;
      }
    }

    hash
  }

  fn _encode(&self, numbers: &Vec<i64>) -> String {
    let mut numberHashInt  = 0;
    let mut count = 100;
    for number in numbers.iter() {
      numberHashInt += (*number % count);
      count += 1;
    }

    let idx = (numberHashInt % (self.alphabet.len() as i64)) as usize;
    let ret = self.alphabet.slice(idx, idx+1).to_string();
    let mut ret_str = ret.clone();

    let mut t_alphabet = self.alphabet.clone();
    for number in numbers.iter() {
      let mut buffer = ret.clone();
      buffer.push_str(self.salt.as_slice());
      buffer.push_str(t_alphabet.as_slice());
      t_alphabet = HashIds::hashids_shuffle(t_alphabet.clone(), buffer.slice(0, t_alphabet.len()).to_string());
      let last = HashIds::hash(*number, t_alphabet.clone());

      ret_str.push_str(last.as_slice());
    }

    if ret_str.len() < self.minHashLength {
      let guardIdx = (numberHashInt + ret_str.clone().into_bytes()[0] as i64) as usize % self.guards.len();
      let guard = self.guards.slice(guardIdx, guardIdx+1).to_string();
      let mut t = guard.clone();
      t.push_str(ret_str.as_slice());
      ret_str = t;

      if ret_str.len() < self.minHashLength {
        let guardIdx = (numberHashInt + ret_str.clone().into_bytes()[2] as i64) as usize % self.guards.len();
        let guard = self.guards.slice(guardIdx, guardIdx+1);
        ret_str.push_str(guard)
      }
    }

    let halfLen = t_alphabet.len() / 2;
    while ret_str.len() < self.minHashLength {
      t_alphabet = HashIds::hashids_shuffle(t_alphabet.clone(), t_alphabet.clone());
      let mut t_ret = "".to_string();
      t_ret.push_str(t_alphabet.slice_from(halfLen));
      t_ret.push_str(ret_str.as_slice());
      t_ret.push_str(t_alphabet.slice(0, halfLen));
      ret_str = t_ret;

      let excess = (ret_str.len() as i64 - self.minHashLength as i64);
      println!("excess = {}", excess);
      if (excess > 0) {
        let start_pos = (excess as i64 / 2) as usize;
        ret_str = ret_str.slice(start_pos, start_pos + self.minHashLength).to_string();
      }
    }

    ret_str
  }
}

fn main() {
//  let ids = HashIds::new("1234".to_string(), 10, DEFAULT_ALPHABET.to_string());
  let ids = HashIds::newWithSalt("this is my salt".to_string());
  println!("{}", ids.alphabet);
  println!("{}", ids.separators);
  let numbers: Vec<i64> = vec![12345];
  let encode = ids.encrypt(&numbers);
  println!("{}", encode);

  let longs = ids.decrypt(encode);
  for s in longs.iter() {
    println!("{}", s);
  }
}
