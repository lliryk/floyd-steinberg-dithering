use std::process;

use clap::Parser;

use floyd_dithering::config::Config;
use floyd_dithering::config::Extension;
use floyd_dithering::bit_map::*;

fn main() {
    
    let config = Config::parse();

    let ext = config.ext().unwrap_or_else(|err| {
        eprintln!("Error while processing file extension: {}", err);
        process::exit(1);
    });

    match ext {
        Extension::BMP => {
            let mut bit_map = BitMap::new(&config.filename).unwrap_or_else(|err| {
                eprintln!("Error while processing bitmap file: {}", err);
                process::exit(1);
            });

            let pallete = config.pallete();

            // Transform the image
            bit_map.dither_floydsteinberg(&pallete, config.bits);

            let size = bit_map.save(config.output.as_path()).unwrap_or_else(|err| {
                eprintln!("Error while saving bitmap file: {}", err);
                process::exit(1);
            });
            
            println!("Wrote {} bytes to {}", size, match config.output.to_str() {
                Some(filename) => { filename },
                None => {""}
            });
            
        }
    }

}