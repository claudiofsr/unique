
// Cargo.toml:
// [dependencies]
// ring = "0.16.20"

use ring::digest::{Algorithm, Context};
use std::fs;
// use data_encoding::HEXLOWER; // data-encoding = "2.3.2"

const HEX: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];

pub trait ExtraProperties {
    fn remove_multiple_whitespace(self) -> String;
    fn remove_first_and_last_char(self) -> String;
    fn count_char(self, ch: char) -> usize;
}

impl ExtraProperties for &str {
    // https://stackoverflow.com/questions/71864137/whats-the-ideal-way-to-trim-extra-spaces-from-a-string
    // Replace multiple whitespace with single whitespace
    // Substituir dois ou mais espaços em branco por apenas um
    fn remove_multiple_whitespace(self) -> String {
        let mut new_str: String = self.to_owned();
        let mut previous_char: char = 'x'; // some non-whitespace character
        new_str.retain(|current_char| {
            let keep: bool = !(previous_char == ' ' && current_char == ' ');
            previous_char = current_char;
            keep
        });
        new_str
    }

    // https://stackoverflow.com/questions/65976432/how-to-remove-first-and-last-character-of-a-string-in-rust
    fn remove_first_and_last_char(self) -> String {
        let mut chars = self.chars();
        chars.next();
        chars.next_back();
        chars.into_iter().collect()
    }

    // line = "|C170|foo|bar|zzz|" --> pipes: '|||||' ; size: 5
    fn count_char(self, ch: char) -> usize {
        let mut new_str: String = self.to_owned();
        new_str.retain(|current_char| current_char == ch);
        //println!("pipes: '{new_str}' ; size: {}", new_str.len());
        new_str.len()
    }
}

pub trait NewProperties {
    fn hex_string(self) -> String;
}

impl NewProperties for &[u8] {
    fn hex_string(self) -> String {
        let mut buf = String::with_capacity(2*self.len());
        for byte in self {
            buf.push(HEX[(*byte as usize)/16]);
            buf.push(HEX[(*byte as usize)%16]);
        }
        buf
    }
}

// https://docs.rs/ring/latest/ring/digest/fn.digest.html
pub fn ring_hash(file_path: &str, algorithm: &'static Algorithm) -> String {

    let bytes: Vec<u8> = match fs::read(file_path) {
        Ok(f) => f,
        Err(_) => file_path.as_bytes().to_vec(),
    };

    let mut context = Context::new(algorithm); // algorithm: SHA256 or SHA512
    context.update(&bytes);
    let result = context.finish();

    //println!("file_path: {file_path} ; bytes: {bytes:?}");
    //println!("result.algorithm(): {:?} ; result: {:?}", result.algorithm(), result);

    let info: &[u8] = result.as_ref();

    // u8 slice in hex representation
    // let hex_str: String = HEXLOWER.encode(info);
    let hex_str: String = info.hex_string();

    //println!("info: {info:?} ; hex_str:{hex_str}");

    hex_str
}

#[cfg(test)]
mod digests {
    use ring::digest::{SHA256, SHA512};
    use super::*;

    // cargo test -- --show-output

    // echo -n test | sha256sum
    // 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08

    #[test]
    fn ring_sha256_sum() {
        let input: &str = "test";
        let hash = ring_hash(input, &SHA256);
        println!("sha256({input}): {hash}");
        let expected_result = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
        assert_eq!(&hash, expected_result);
    }

    // echo -n test | sha512sum
    // 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08

    #[test]
    fn ring_sha512_sum() {
        let input: &str = "test";
        let hash = ring_hash(input, &SHA512);
        println!("sha512({input}): {hash}");
        let expected_result = "ee26b0dd4af7e749aa1a8ee3c10ae9923f618980772e473f8819a5d4940e0db27ac185f8a0e1d5f84f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8ff";
        assert_eq!(&hash, expected_result);
    }

    #[test]
    fn test_remove_multiple_whitespace() {
        let strings: Vec<&str> = vec![
            "teste",
            " teste", "teste ", " teste ",
            "  teste", "teste  ", "  teste  ",
            "tes te", "tes  te", "tes   te",
            " tes te", "tes  te ", " tes  te ",
            "  tes te", "tes  te  ", "  tes  te  ",
        ];
        for string in strings {
            let s = ["'", string, "'"].concat();
            println!("{:13} --> '{}'", s, string.remove_multiple_whitespace());
        }
        let s1 = "tes  te".remove_multiple_whitespace();
        let s2 = " tes  te".remove_multiple_whitespace();
        let s3 = "tes  te ".remove_multiple_whitespace();
        let s4 = " tes  te ".remove_multiple_whitespace();
        let s5 = "  tes  te".remove_multiple_whitespace();
        let s6 = "tes  te  ".remove_multiple_whitespace();
        let s7 = "  tes  te  ".remove_multiple_whitespace();

        assert_eq!(s1, "tes te");
        assert_eq!(s2, " tes te");
        assert_eq!(s3, "tes te ");
        assert_eq!(s4, " tes te ");
        assert_eq!(s5, " tes te");
        assert_eq!(s6, "tes te ");
        assert_eq!(s7, " tes te ");
    }
}
