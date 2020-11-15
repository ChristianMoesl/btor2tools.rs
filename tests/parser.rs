use btor2tools::Btor2Parser;
use std::{env::current_dir, fs::read_dir};

#[test]
fn parse_btor2_examples() {
    let examples_dir = current_dir().unwrap().join("btor2tools/examples/btorsim");

    read_dir(&examples_dir)
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.path().to_str().unwrap().ends_with(".btor2"))
        .for_each(|entry| {
            let path = entry.path();

            println!("parse: {}", path.display());

            Btor2Parser::new()
                .read_lines(&path)
                .unwrap()
                .for_each(|line| {
                    // panics of one something is not printable
                    println!("{:?}", line);
                })
        });
}
