use std::{env, process};

use moviebarcode::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Error reading args: {:?}", err);
        process::exit(1);
    });

    moviebarcode::run(&config).unwrap_or_else(|err| {
        println!("Error generating barcode: {:?}", err);
        process::exit(1);
    });
}
