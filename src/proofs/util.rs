use num::{BigInt, Num};

pub fn convert_hex_to_dec(hex_str: String) -> String {
    BigInt::from_str_radix(hex_str.as_str().strip_prefix("0x").unwrap(), 16)
        .unwrap()
        .to_string()
}
