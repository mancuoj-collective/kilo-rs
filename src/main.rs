use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{Read, stdin};

fn main() {
    enable_raw_mode().unwrap();

    for b in stdin().bytes() {
        match b {
            Ok(b) => {
                let c = b as char;

                if c.is_control() {
                    println!("Binary: {b:08b}, ASCII: {b:03} \r");
                } else {
                    println!("Binary: {b:08b}, ASCII: {b:03}, Character: {c:#?}\r");
                }

                if c == 'q' {
                    break;
                }
            }
            Err(err) => println!("{err}"),
        }
    }

    disable_raw_mode().unwrap();
}
