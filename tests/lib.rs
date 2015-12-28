extern crate hashids;

use hashids::HashIds;

#[test]
fn it_works_1() {
  let ids_some = HashIds::new_with_salt("this is my salt".to_string());
  let ids = match ids_some {
    Ok(v) => { v }
    Err(_) => {
      println!("error");
      return;
    }
  };

  let numbers: Vec<i64> = vec![12345];
  let encode = ids.encode(&numbers);
  let longs = ids.decode(encode.clone());

  assert_eq!(numbers, longs);
}

#[test]
fn it_works_2() {
  let ids_some = HashIds::new_with_salt("this is my salt".to_string());
  let ids = match ids_some {
    Ok(v) => { v }
    Err(_) => {
      println!("error");
      return;
    }
  };

  let numbers: Vec<i64> = vec![12345];
  let encode = ids.encode(&numbers);

  let ids_some2 = HashIds::new_with_salt("this is my pepper".to_string());
  let ids2 = match ids_some2 {
    Ok(v) => { v }
    Err(_) => {
      println!("error");
      return;
    }
  };

  let longs = ids2.decode(encode.clone());

  assert_eq!(longs.len(), 0);
}

#[test]
fn it_works_3() {
  let ids_some = HashIds::new_with_salt("this is my salt".to_string());
  let ids = match ids_some {
    Ok(v) => { v }
    Err(_) => {
      println!("error");
      return;
    }
  };

  let numbers: Vec<i64> = vec![683, 94108, 123, 5];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "aBMswoO2UB3Sj");
}

#[test]
fn it_works_4() {
  let ids_some = HashIds::new_with_salt_and_min_length("this is my salt".to_string(), 8);
  let ids = match ids_some {
    Ok(v) => { v }
    Err(_) => {
      println!("error");
      return;
    }
  };

  let numbers: Vec<i64> = vec![1];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "gB0NV05e");
}

#[test]
fn it_works_5() {
  let ids_some = HashIds::new("this is my salt".to_string(), 0, "0123456789abcdef".to_string());
  let ids = match ids_some {
    Ok(v) => { v }
    Err(_) => {
      println!("error");
      return;
    }
  };

  let numbers: Vec<i64> = vec![1234567];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "b332db5");
}

#[test]
fn it_works_6() {
  let ids_some = HashIds::new_with_salt("this is my salt".to_string());
  let ids = match ids_some {
    Ok(v) => { v }
    Err(_) => {
      println!("error");
      return;
    }
  };

  let numbers: Vec<i64> = vec![5, 5, 5, 5];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "1Wc8cwcE");
}

#[test]
fn it_works_7() {
  let ids_some = HashIds::new_with_salt("this is my salt".to_string());
  let ids = match ids_some {
    Ok(v) => { v }
    Err(_) => {
      println!("error");
      return;
    }
  };

  let numbers: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  let encode = ids.encode(&numbers);

  assert_eq!(encode, "kRHnurhptKcjIDTWC3sx");
}

#[test]
fn it_works_8() {
  let ids_some = HashIds::new_with_salt("this is my salt".to_string());
  let ids = match ids_some {
    Ok(v) => { v }
    Err(_) => {
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

  assert_eq!(encode_1, "NV");
  assert_eq!(encode_2, "6m");
  assert_eq!(encode_3, "yD");
  assert_eq!(encode_4, "2l");
  assert_eq!(encode_5, "rD");
}
