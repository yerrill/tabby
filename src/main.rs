mod codegen;
mod filetype;
mod state;

use codegen::{Generation, Python};
use filetype::{Filetype, JsonFileType};
use std::{error::Error, process};

fn example() -> Result<(), Box<dyn Error>> {
    let file = std::fs::read_to_string("test.json").unwrap();

    // let file = "[{ \"a\": 1, \"b\": 2}, { \"a\": 1.1, \"b\": 2}]".to_string();

    let json = JsonFileType::new(file.as_str()).unwrap();

    let obj = json.to_object();

    let code = Python::generate(obj);

    println!("{}", code);

    Ok(())
}

fn main() {
    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
