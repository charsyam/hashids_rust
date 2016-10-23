#![crate_name = "hashids"]

extern crate regex;

use std::collections::HashMap;
use regex::Regex;
use std::fmt::{self, Debug};

const DEFAULT_ALPHABET: &'static str =  "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
const DEFAULT_SEPARATORS: &'static str = "cfhistuCFHISTU";
const SEPARTOR_DIV: f32 = 3.5;
const GUARD_DIV: u32 = 12;
const MIN_ALPHABET_LENGTH: usize = 16;

pub enum HashIdsError { InvalidAlphabetLength }

pub struct HashIds {
    salt: String,
    pub alphabet: String,
    separators: String,
    min_hash_length: usize,
    guards: String
}

impl Debug for HashIds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "salt: {}\n{}",
                 self.salt,
                 self.alphabet,
        )
    }
}

impl Debug for HashIdsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      writeln!(f, "InvalidAlphabetLength")
    }
}

impl HashIds {
    pub fn new(salt: String, min_hash_length: usize, alphabet: String) -> Result<HashIds, HashIdsError>  {
        let min_length = HashIds::get_min_hash_length(min_hash_length);
        let unique_alphabet = HashIds::get_unique_alphabet(alphabet);

        if unique_alphabet.len() < MIN_ALPHABET_LENGTH {
            return Err(HashIdsError::InvalidAlphabetLength);
        }

        let (t_separators, mut t_alphabet) = HashIds::get_separators(unique_alphabet);
        let mut shuffled_separators = HashIds::hashids_shuffle(t_separators.clone(), salt.clone());

        if HashIds::need_manipulate(shuffled_separators.len(), t_alphabet.len()) == true {
            let mut seps_len =  ((t_alphabet.len() as f32) / SEPARTOR_DIV) as usize;
            if seps_len == 1 {
                seps_len += 1;
            }

            if seps_len > shuffled_separators.len() {
                let diff = seps_len - shuffled_separators.len();

                shuffled_separators.push_str(&t_alphabet[..diff]);
                t_alphabet = t_alphabet[diff..].to_string();
            } else {
                shuffled_separators = shuffled_separators[..seps_len].to_string();
            }
        }

        let mut shuffled_alphabet = HashIds::hashids_shuffle(t_alphabet.clone(), salt.clone());
        let guard_count = (shuffled_alphabet.len() as f32 / GUARD_DIV as f32).ceil() as usize;

        let t_guards;

        if shuffled_alphabet.len() < 3 {
            t_guards = shuffled_separators[..guard_count].to_string();
            shuffled_separators = shuffled_separators[guard_count..].to_string();
        } else {
            t_guards = shuffled_alphabet[..guard_count].to_string();
            shuffled_alphabet = shuffled_alphabet[guard_count..].to_string();
        }

        Ok(HashIds {
            salt: salt,
            min_hash_length: min_length,
            guards: t_guards,
            separators: shuffled_separators,
            alphabet: shuffled_alphabet
        })
    }

    pub fn new_with_salt_and_min_length(salt: String, min_hash_length: usize) -> Result<HashIds, HashIdsError> {
        HashIds::new(salt, min_hash_length, DEFAULT_ALPHABET.to_string())
    }

    pub fn new_with_salt(salt: String) -> Result<HashIds, HashIdsError> {
        HashIds::new_with_salt_and_min_length(salt, 0)
    }


    fn need_manipulate(slen: usize, alen: usize) -> bool {
        if slen <= 0 || (((alen/slen) as f32)> SEPARTOR_DIV) {
            return true;
        }

        false
    }

    fn get_min_hash_length(length: usize) -> usize {
        if length > 0 {
            return length;
        }

        0
    }

    fn get_separators(alphabet: String) -> (String, String) {
        HashIds::get_non_duplicated_string(DEFAULT_SEPARATORS.to_string(), alphabet)
    }

    fn hashids_shuffle(alphabet: String, salt: String) -> String {
        if salt.len() <= 0 {
            return alphabet;
        }

        let salt_len = salt.len();
        let arr = salt.as_bytes();
        let len = alphabet.len();
        let mut bytes = alphabet.into_bytes();
        let mut shuffle = &mut bytes[..];

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
        for i in 0..len {
            shuffled_alphabet.push(shuffle[i] as char);
        }

        shuffled_alphabet
    }

    fn get_non_duplicated_string(separators: String, alphabet: String) -> (String, String) {
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

    pub fn encode_hex(&self, hex: String) -> String {
        let regex1 = Regex::new(r"^[0-9a-fA-F]+$").unwrap();
        if regex1.is_match(&hex.to_string()) == false {
            return String::new();
        }

        let mut numbers: Vec<i64> = Vec::new();
        let regex2 = Regex::new(r"[\w\W]{1,12}").unwrap();
        for matcher in regex2.find_iter(&hex.to_string()) {
            let mut num = String::new();
            num.push('1');
            num.push_str(&hex[matcher.0..matcher.1]);
            let v: i64 = i64::from_str_radix(&num.to_string(), 16).unwrap();
            numbers.push(v);
        }

        self.encode(&numbers)
    }

    pub fn decode_hex(&self, hash: String) -> String {
        let mut ret = String::new();
        let numbers = self.decode(hash);
        for number in numbers.iter() {
            let r = format!("{:x}", number);
            ret.push_str(&r[1..]);
        }

        ret
    }

    pub fn encode(&self, numbers: &Vec<i64>) -> String {
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

    pub fn decode(&self, hash: String) -> Vec<i64> {
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
        regexp.push_str(&self.guards[..]);
        regexp.push(']');

        let re = Regex::new(&regexp[..]).unwrap();
        let t_hash = re.replace_all(&hash[..], " ");

        let split1: Vec<&str> = t_hash[..].split_whitespace().collect();
        let mut i = 0;

        let len = split1.len();
        if len == 3 || len == 2 {
            i = 1;
        }
        let mut hash_breakdown = split1[i].to_string();

        let lottery = hash_breakdown[0..1].to_string();
        hash_breakdown = hash_breakdown[1..].to_string();

        let mut regexp2 = String::new();
        regexp2.push('[');
        regexp2.push_str(&self.separators[..]);
        regexp2.push(']');

        let re2 = Regex::new(&regexp2[..]).unwrap();
        hash_breakdown = re2.replace_all(&hash_breakdown[..], " ");

        let split2: Vec<&str> = hash_breakdown[..].split_whitespace().collect();

        let mut alphabet = self.alphabet.clone();

        let mut ret: Vec<i64> = Vec::new();

        for s in split2 {
            let sub_hash = s.to_string();
            let mut buffer = String::new();
            buffer.push_str(&lottery[..]);
            buffer.push_str(&self.salt[..]);
            buffer.push_str(&alphabet.clone()[..]);

            let alpha_len = alphabet.len();
            alphabet = HashIds::hashids_shuffle(alphabet, buffer[0..alpha_len].to_string());
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
            let pow_size = (len - i - 1) as u32;
            number += (pos * alpha_len.pow(pow_size)) as i64;
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
            let mut t = alphabet[idx..idx+1].to_string();
            t.push_str(&hash[..]);
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
            number_hash_int += *number % count;
            count += 1;
        }

        let idx = (number_hash_int % (self.alphabet.len() as i64)) as usize;
        let ret = self.alphabet[idx..idx+1].to_string();
        let mut ret_str = ret.clone();

        let mut t_alphabet = self.alphabet.clone();
        let mut i = 0;
        let len = self.separators.len() as i64;
        let last_len = count - 100;
        for number in numbers.iter() {
            let mut buffer = ret.clone();
            buffer.push_str(&self.salt[..]);
            buffer.push_str(&t_alphabet[..]);
            t_alphabet = HashIds::hashids_shuffle(t_alphabet.clone(), buffer[0..t_alphabet.len()].to_string());
            let last = HashIds::hash(*number, t_alphabet.clone());

            ret_str.push_str(&last[..]);

            if (i + 1) < last_len {
            let mut v = *number % (last.as_bytes()[0] as i64 + i);
            v = v % len;
            ret_str.push(self.separators.as_bytes()[v as usize] as char);
            }
            i += 1;
        }

        if ret_str.len() < self.min_hash_length {
            let guard_idx = (number_hash_int + ret_str.clone().into_bytes()[0] as i64) as usize % self.guards.len();
            let guard = self.guards[guard_idx..guard_idx+1].to_string();
            let mut t = guard.clone();
            t.push_str(&ret_str[..]);
            ret_str = t;

            if ret_str.len() < self.min_hash_length {
            let guard_idx = (number_hash_int + ret_str.clone().into_bytes()[2] as i64) as usize % self.guards.len();
            ret_str.push_str(&self.guards[guard_idx..guard_idx+1]);
            }
        }

        let half_len = t_alphabet.len() / 2;
        while ret_str.len() < self.min_hash_length {
            t_alphabet = HashIds::hashids_shuffle(t_alphabet.clone(), t_alphabet.clone());
            let mut t_ret = "".to_string();
            t_ret.push_str(&t_alphabet[half_len..]);
            t_ret.push_str(&ret_str[..]);
            t_ret.push_str(&t_alphabet[0..half_len]);
            ret_str = t_ret;

            let excess = ret_str.len() as i64 - self.min_hash_length as i64;
            if excess > 0 {
            let start_pos = (excess as i64 / 2) as usize;
            ret_str = ret_str[start_pos..start_pos + self.min_hash_length].to_string();
            }
        }

        ret_str
    }
}
