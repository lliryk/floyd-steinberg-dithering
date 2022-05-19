use std::env;
use std::process;
use std::path::Path;

use floyd_dithering::config::Config;
use floyd_dithering::config::Extension;
use floyd_dithering::bit_map::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Encountered an error processing config: {}", err);
        process::exit(1);
    });

    println!("{:?}", config);

    match config.extension {
        Extension::BMP => {
            let mut bit_map = BitMap::new(&config.filename).unwrap_or_else(|err| {
                eprintln!("Error while processing bitmap file: {}", err);
                process::exit(1);
            });
            bit_map.debug();

            let pallete = Pallete::new(&["white", "black", "red", "green", "blue"]);

            // Transform the image
            bit_map.dither_floydsteinberg(&pallete, 3);

            let size = bit_map.save(Path::new("output/edited.bmp")).unwrap_or_else(|err| {
                eprintln!("Error while saving bitmap file: {}", err);
                process::exit(1);
            });
            
            println!("Wrote {} bytes", size);
            
        }
    }

}