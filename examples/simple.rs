extern crate hashids;
use hashids::HashIds;

fn main() {
    let ids = HashIds::new_with_salt("this is my salt".to_string()).unwrap();
    let numbers: Vec<i64> = vec!(21312321, 34);
    let encode = ids.encode(&numbers);

    println!("{}", encode);
}