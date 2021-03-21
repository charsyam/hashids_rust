# HashIds for Rust

A Rust port of the JavaScript *hashids* implementation. It generates YouTube-like hashes from one or many numbers. Use hashids when you do not want to expose your database ids to the user. Website: http://www.hashids.org/

## What is it?

hashids (Hash ID's) creates short, unique, decodeable hashes from unsigned (long) integers.

It was designed for websites to use in URL shortening, tracking stuff, or making pages private (or at least unguessable).

This algorithm tries to satisfy the following requirements:

1. Hashes must be unique and decodable.
2. They should be able to contain more than one integer (so you can use them in complex or clustered systems).
3. You should be able to specify minimum hash length.
4. Hashes should not contain basic English curse words (since they are meant to appear in public places - like the URL).

Instead of showing items as `1`, `2`, or `3`, you could show them as `U6dc`, `u87U`, and `HMou`.
You don't have to store these hashes in the database, but can encode + decode on the fly.

All (long) integers need to be greater than or equal to zero.

## Usage

#### Import the package
```rust
extern crate hashids;
use hashids::HashIds;
```

see tests/lib.rs

#### Encrypting one number

You can pass a unique salt value so your hashes differ from everyone else's. I use "this is my salt" as an example.

```rust
let ids_some = HashIds::new_with_salt("this is my salt".to_string());
let ids = match ids_some {
  Ok(v) => { v }
  Err(e) => {
    println!("error");
    return;
  }
};

let numbers: Vec<i64> = vec![12345];
let encode = ids.encode(&numbers);
```

`hash` is now going to be:

	NkK9

#### Decrypting

Notice during decoding, same salt value is used:

```rust
let longs = ids.decode("NkK9".to_string());
for s in longs.iter() {
  println!("longs: {}", s);
}
```

`numbers` is now going to be:

	[ 12345 ]

#### Decrypting with different salt

Decryption will not work if salt is changed:

```rust
let ids_some = HashIds::new_with_salt("this is my salt".to_string());
let ids = match ids_some {
  Ok(v) => { v }
  Err(e) => {
    println!("error");
    return;
  }
};

let numbers = ids.decode("NkK9");
```

`numbers` is now going to be:

	[]

#### Encrypting several numbers

```Rust
let ids_some = HashIds::new_with_salt("this is my salt".to_string());
let ids = match ids_some {
  Ok(v) => { v }
  Err(e) => {
    println!("error");
    return;
  }
};

let numbers: Vec<i64> = vec![683, 94108, 123, 5];
let encode = ids.encode(&numbers);
```

`hash` is now going to be:

	aBMswoO2UB3Sj

#### Decrypting is done the same way

```rust
let ids_some = HashIds::new_with_salt("this is my salt".to_string());
let ids = match ids_some {
  Ok(v) => { v }
  Err(e) => {
    println!("error");
    return;
  }
};

let longs = ids.decode("NkK9".to_string());
for s in longs.iter() {
  println!("longs: {}", s);
}
```

`numbers` is now going to be:

	[ 683, 94108, 123, 5 ]

#### Encrypting and specifying minimum hash length

Here we encode integer 1, and set the minimum hash length to **8** (by default it's **0** -- meaning hashes will be the shortest possible length).

```rust
let ids_some = HashIds::new_with_salt_and_min_length("this is my salt".to_string(), 8);
let ids = match ids_some {
  Ok(v) => { v }
  Err(e) => {
    println!("error");
    return;
  }
};

let numbers : Vec<i64> = vec![1];
let encode = ids.encode(&numbers);
```

`hash` is now going to be:

	gB0NV05e

#### Decrypting

```rust
let ids_some = HashIds::new_with_salt_and_min_length("this is my salt".to_string(), 8);
let ids = match ids_some {
  Ok(v) => { v }
  Err(e) => {
    println!("error");
    return;
  }
};

let numbers = ids.decode("gB0NV05e")
```

`numbers` is now going to be:

	[ 1 ]

#### Specifying custom hash alphabet

Here we set the alphabet to consist of only four letters: "0123456789abcdef"

```rust
let ids_some = HashIds::new("this is my salt".to_string(), 0, "0123456789abcdef".to_string());
let ids = match ids_some {
  Ok(v) => { v }
  Err(e) => {
    println!("error");
    return;
  }
};


let numbers : Vec<i64> = vec![1234567];
hashids.encode(&numbers);
```

`hash` is now going to be:

	b332db5

## Randomness

The primary purpose of hashids is to obfuscate ids. It's not meant or tested to be used for security purposes or compression.
Having said that, this algorithm does try to make these hashes unguessable and unpredictable:

#### Repeating numbers

```rust
let ids_some = HashIds::new_with_salt("this is my salt".to_string());
let ids = match ids_some {
  Ok(v) => { v }
  Err(e) => {
    println!("error");
    return;
  }
};

let numbers: Vec<i64> = vec![5, 5, 5, 5];
let encode = ids.encode(&numbers);
```

You don't see any repeating patterns that might show there's 4 identical numbers in the hash:

	1Wc8cwcE

Same with incremented numbers:

```rust
let ids_some = HashIds::new_with_salt("this is my salt".to_string());
let ids = match ids_some {
  Ok(v) => { v }
  Err(e) => {
    println!("error");
    return;
  }
};

let numbers: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
let encode = ids.encode(&numbers);
```

`hash` will be :

	kRHnurhptKcjIDTWC3sx

### Incrementing number hashes:

```rust
let ids_some = HashIds::new_with_salt("this is my salt".to_string());
let ids = match ids_some {
  Ok(v) => { v }
  Err(e) => {
    println!("error");
    return;
  }
};

let numbers_1: Vec<i64> = vec![1];
let encode_1 = ids.encode(&numbers_1);

let numbers_2: Vec<i64> = vec![2];
let encode_2 = ids.encode(&numbers_2);

let numbers_3: Vec<i64> = vec![3];
let encode_3 = ids.encode(&numbers_3);

let numbers_4: Vec<i64> = vec![4];
let encode_4 = ids.encode(&numbers_4);

let numbers_5: Vec<i64> = vec![5];
let encode_5 = ids.encode(&numbers_5);
```

## Bad hashes

I wrote this class with the intent of placing these hashes in visible places - like the URL. If I create a unique hash for each user, it would be unfortunate if the hash ended up accidentally being a bad word. Imagine auto-creating a URL with hash for your user that looks like this - `http://example.com/user/a**hole`

Therefore, this algorithm tries to avoid generating most common English curse words with the default alphabet. This is done by never placing the following letters next to each other:

	c, C, s, S, f, F, h, H, u, U, i, I, t, T

## Contact

Follow me [@charsyam](https://twitter.com/charsyam), [@IvanAkimov](http://twitter.com/ivanakimov)

## License

MIT License. See the `LICENSE` file.
