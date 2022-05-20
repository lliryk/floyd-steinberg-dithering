use std::process;

use clap::Parser;

use floyd_dithering::config::Config;
use floyd_dithering::config::Extension;
use floyd_dithering::bit_map::*;

fn main() {
    
    let config = Config::parse();

    println!("{:?}", config);

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
            bit_map.debug();

            let pallete = config.pallete();

            // Transform the image
            bit_map.dither_floydsteinberg(&pallete, 3);

            let size = bit_map.save(config.output.as_path()).unwrap_or_else(|err| {
                eprintln!("Error while saving bitmap file: {}", err);
                process::exit(1);
            });
            
            println!("Wrote {} bytes", size);
            
        }
    }

}