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
  min_hash_length: usize,
  guards: String 
}

impl HashIds {

  pub fn new(salt: String, min_hash_length: usize, alphabet: String) -> HashIds {
    use std::num::Float;

    let min_length = HashIds::get_min_hash_length(min_hash_length);
    let unique_alphabet = HashIds::get_unique_alphabet(alphabet);

    if unique_alphabet.len() < ::MIN_ALPHABET_LENGTH {
      //Error
    }

    let (t_separators, mut t_alphabet) = HashIds::get_separators(unique_alphabet);
    let mut shuffled_separators = HashIds::hashids_shuffle(t_separators.clone(), salt.clone());

    if HashIds::need_manipulate(shuffled_separators.len(), t_alphabet.len()) == true {
      let mut seps_len =  ((t_alphabet.len() as f32) / ::SEP_DIV) as usize;
      if seps_len == 1 {
        seps_len += 1;
      }

      if seps_len > shuffled_separators.len() {
        let diff = seps_len - shuffled_separators.len();
        shuffled_separators.push_str(t_alphabet.slice(0,diff));
        t_alphabet = t_alphabet.slice_from(diff).to_string();
      } else {
        shuffled_separators = shuffled_separators.slice(0, seps_len).to_string();
      }
    }

    let mut shuffled_alphabet = HashIds::hashids_shuffle(t_alphabet.clone(), salt.clone());
    let guard_count = (shuffled_alphabet.len() as f32 / ::GUARD_DIV as f32).ceil() as usize;

    let mut t_guards;

    if guard_count < 3 {
      t_guards = shuffled_separators.slice(0, guard_count).to_string();
      shuffled_separators = shuffled_separators.slice_from(guard_count).to_string(); 
    } else {
      t_guards = shuffled_alphabet.slice(0, guard_count).to_string();
      shuffled_alphabet = shuffled_alphabet.slice_from(guard_count).to_string(); 
    }
    
    HashIds {
      salt: salt,
      min_hash_length: min_length,
      guards: t_guards,
      separators: shuffled_separators,
      alphabet: shuffled_alphabet
    }
  }

  pub fn new_with_salt_and_min_length(salt: String, min_hash_length: usize) -> HashIds {
    HashIds::new(salt, min_hash_length, DEFAULT_ALPHABET.to_string())
  }

  pub fn new_with_salt(salt: String) -> HashIds {
    HashIds::new_with_salt_and_min_length(salt, 0)
  }


  fn need_manipulate(slen: usize, alen: usize) -> bool {
    if slen <= 0 && (((alen/slen) as f32)> ::SEP_DIV) {
      return true;
    }

    false
  }

  fn get_min_hash_length(length: usize) -> usize {
    if length > 0 {
      return length;
    } else {
      return 0;
    }
  }

  fn get_separators(alphabet: String) -> (String, String) {
    HashIds::get_non_duplicated_string(::DEFAULT_SEPARATORS.to_string(), alphabet)
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
    let mut v: usize = 0;
    let mut p: usize = 0;

    while i > 0 {
      v %= salt_len;
      let t = arr[v] as usize;
      p += t;
      let j = (t + v + p) % i;

      shuffle.swap(i,j);

      i=i-1;
      v=v+1; 
    }

    let mut shuffled_alphabet = String::with_capacity(len);
    for i in (0..len) {
      shuffled_alphabet.push(shuffle[i] as char);
    }

    shuffled_alphabet
  }

  fn get_non_duplicated_string(separators: String, alphabet: String) -> (String, String) {
    use std::collections::HashMap;
    let mut check_separator_map = HashMap::new();
    let mut check_alphabet_map = HashMap::new();

    let mut modified_separators = String::new();
    let mut modified_alphabet = String::new();
    
    for c in separators.chars() {
      check_separator_map.insert(c, 1);
    }

    for c in alphabet.chars() {
      check_alphabet_map.insert(c, 1);
    }

    for c in separators.chars() {
      if check_alphabet_map.contains_key(&c) {
        modified_separators.push(c);
      }
    }
    
    for c in alphabet.chars() {
      if !check_separator_map.contains_key(&c) {
        modified_alphabet.push(c);
      }
    }
    
    (modified_separators, modified_alphabet)
  }

  fn get_unique_alphabet(alphabet: String) -> String {
    use std::collections::HashMap;

    let mut unique_alphabet: String = String::new();
    let mut check_map = HashMap::new();
    
    for c in alphabet.chars() {
      if !check_map.contains_key(&c) {
        unique_alphabet.push(c);
        check_map.insert(c, 1);
      }
    }

    unique_alphabet
  }

  fn encrypt(&self, numbers: &Vec<i64>) -> String {
    return self.encode(numbers);
  }

  fn decrypt(&self, hash: String) -> Vec<i64> {
    return self.decode(hash);
  }

  fn encode_hex(&self, hex: String) -> String {
    use regex::Regex;
    use std::num;
    let regex1 = Regex::new(r"^[0-9a-fA-F]+$").unwrap();
    if regex1.is_match(hex.as_slice()) == false {
      return String::new();
    }

    let mut numbers: Vec<i64> = Vec::new();
    let regex2 = Regex::new(r"[\w\W]{1,12}").unwrap();
    for matcher in regex2.find_iter(hex.as_slice()) {
      let mut num = String::new();
      num.push('1');
      num.push_str(hex.slice(matcher.0, matcher.1));
      let v: i64 = num::from_str_radix(num.as_slice(), 16).unwrap();
      numbers.push(v);
    }
    
    self.encode(&numbers)
  }

  fn decode_hex(&self, hash: String) -> String {
    use std::fmt;
    let mut ret = String::new();
    let numbers = self.decode(hash);
    for number in numbers.iter() {
      let r = format!("{:x}", number);
      ret.push_str(r.slice_from(1));
    }

    ret
  }

  fn encode(&self, numbers: &Vec<i64>) -> String {
    if numbers.len() == 0 {
      return "".to_string();
    }

    for number in numbers.iter() {
      if *number > 9007199254740992 {
        return "".to_string();
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
    let mut hash_breakdown = split[i].to_string();

    let lottery = hash_breakdown.slice(0,1).to_string();

    hash_breakdown = hash_breakdown.slice_from(1).to_string();
    
    let mut regexp2 = String::new();
    regexp2.push('[');
    regexp2.push_str(self.separators.as_slice());
    regexp2.push(']');

    let re2 = Regex::new(regexp2.as_slice()).unwrap();
    hash_breakdown  = re2.replace_all(hash_breakdown.as_slice(), " ");

    split = hash_breakdown.as_slice().split_str(" ").collect();

    let mut alphabet = self.alphabet.clone();

    let mut ret: Vec<i64> = Vec::new();

    for s in split.iter() {
      let sub_hash = s.to_string();
      let mut buffer = String::new();
      buffer.push_str(lottery.as_slice());
      buffer.push_str(self.salt.as_slice());
      buffer.push_str(alphabet.clone().as_slice());

      let alpha_len = alphabet.len();
      alphabet = HashIds::hashids_shuffle(alphabet, buffer.slice(0, alpha_len).to_string());
      ret.push(HashIds::unhash(sub_hash, alphabet.clone()));
    }

    let check_hash = self._encode(&ret);
    if check_hash != hash {
      return Vec::new();
    }

    ret
  }

  fn index_of(input :&[u8], v: u8) -> i64 {
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
    let mut i: usize = 0;
    loop {
      if i >= len {
        break;
      }

      let v = input_slice[i] as usize;
      let pos = HashIds::index_of(alpha_slice, v as u8);
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

    loop {
      let idx = (t_in % len) as usize;
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
    let mut number_hash_int  = 0;
    let mut count = 100;
    for number in numbers.iter() {
      number_hash_int += (*number % count);
      count += 1;
    }

    let idx = (number_hash_int % (self.alphabet.len() as i64)) as usize;
    let ret = self.alphabet.slice(idx, idx+1).to_string();
    let mut ret_str = ret.clone();

    let mut t_alphabet = self.alphabet.clone();
    let mut i = 0;
    let len = self.separators.len() as i64;
    let last_len = count - 100;
    for number in numbers.iter() {
      let mut buffer = ret.clone();
      buffer.push_str(self.salt.as_slice());
      buffer.push_str(t_alphabet.as_slice());
      t_alphabet = HashIds::hashids_shuffle(t_alphabet.clone(), buffer.slice(0, t_alphabet.len()).to_string());
      let last = HashIds::hash(*number, t_alphabet.clone());

      ret_str.push_str(last.as_slice());
      
      if (i + 1) < last_len { 
        let mut v = *number % (last.as_bytes()[0] as i64 + i);
        v = v % len;
        ret_str.push(self.separators.as_bytes()[v as usize] as char);
      }
      i += 1;
    }

    if ret_str.len() < self.min_hash_length {
      let guard_idx = (number_hash_int + ret_str.clone().into_bytes()[0] as i64) as usize % self.guards.len();
      let guard = self.guards.slice(guard_idx, guard_idx+1).to_string();
      let mut t = guard.clone();
      t.push_str(ret_str.as_slice());
      ret_str = t;

      if ret_str.len() < self.min_hash_length {
        let guard_idx = (number_hash_int + ret_str.clone().into_bytes()[2] as i64) as usize % self.guards.len();
        let guard = self.guards.slice(guard_idx, guard_idx+1);
        ret_str.push_str(guard);
      }
    }

    let half_len = t_alphabet.len() / 2;
    while ret_str.len() < self.min_hash_length {
      t_alphabet = HashIds::hashids_shuffle(t_alphabet.clone(), t_alphabet.clone());
      let mut t_ret = "".to_string();
      t_ret.push_str(t_alphabet.slice_from(half_len));
      t_ret.push_str(ret_str.as_slice());
      t_ret.push_str(t_alphabet.slice(0, half_len));
      ret_str = t_ret;

      let excess = ret_str.len() as i64 - self.min_hash_length as i64;
      if (excess > 0) {
        let start_pos = (excess as i64 / 2) as usize;
        ret_str = ret_str.slice(start_pos, start_pos + self.min_hash_length).to_string();
      }
    }

    ret_str
  }
}

fn main() {
//  let ids = HashIds::new("1234".to_string(), 10, DEFAULT_ALPHABET.to_string());
  let ids = HashIds::new_with_salt("this is my salt".to_string());
  println!("{}", ids.alphabet);
  println!("{}", ids.separators);
  let numbers: Vec<i64> = vec![12345];
  let encode = ids.encrypt(&numbers);
  println!("{}", encode);

  let longs = ids.decrypt(encode.clone());
  for s in longs.iter() {
    println!("longs: {}", s);
  }

  let ids3 = HashIds::new_with_salt("this is my pepper".to_string());
  let longs2 = ids3.decrypt(encode.clone());
  for s in longs2.iter() {
    println!("bad longs: {}", s);
  }

  let ids2 = HashIds::new_with_salt("this is my salt".to_string());
  let i = ids2.encode_hex("1234567890abcdef".to_string());
  println!("{}", i);

  let o = ids2.decode_hex(i);
  println!("{}", o);

  let ids3 = HashIds::new_with_salt("this is my salt".to_string());
  let numbers3: Vec<i64> = vec![683, 94108, 123, 5];
  let encode3 = ids3.encrypt(&numbers3);
  println!("ids3 = {}", encode3);

  let ids4 = HashIds::new_with_salt("this is my salt".to_string());
  let numbers4: Vec<i64> = vec![5, 5, 5, 5];
  let encode4 = ids4.encrypt(&numbers4);
  println!("ids4 = {}", encode4);

  let ids5 = HashIds::new_with_salt("this is my salt".to_string());
  let numbers5: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  let encode5 = ids5.encrypt(&numbers5);
  println!("ids5 = {}", encode5);

  let ids6 = HashIds::new_with_salt("this is my salt".to_string());
  let numbers_1: Vec<i64> = vec![1];
  let encode_1 = ids6.encrypt(&numbers_1);
  println!("encode_1 = {}", encode_1);
  let numbers_2: Vec<i64> = vec![2];
  let encode_2 = ids6.encrypt(&numbers_2);
  println!("encode_2 = {}", encode_2);
  let numbers_3: Vec<i64> = vec![3];
  let encode_3 = ids6.encrypt(&numbers_3);
  println!("encode_3 = {}", encode_3);
  let numbers_4: Vec<i64> = vec![4];
  let encode_4 = ids6.encrypt(&numbers_4);
  println!("encode_4 = {}", encode_4);
  let numbers_5: Vec<i64> = vec![5];
  let encode_5 = ids6.encrypt(&numbers_5);
  println!("encode_5 = {}", encode_5);

  let ids7 = HashIds::new_with_salt_and_min_length("this is my salt".to_string(), 8);
  let numbers7 : Vec<i64> = vec![1];
  let encode7 = ids7.encrypt(&numbers_1);
  println!("ids7 = {}", encode7);

  println!("ids7 decrypt = {}", ids7.decrypt(encode7)[0]);
}
