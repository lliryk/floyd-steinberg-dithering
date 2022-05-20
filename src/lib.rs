pub mod pixels;
pub mod bit_map;

pub mod config {
    use std::path::PathBuf;
    use std::path::Path;

    #[derive(Debug)]
    pub enum Extension {
        BMP,
    }

    #[derive(Debug)]
    pub struct Config {
        pub filename: std::path::PathBuf,
        pub extension: Extension,
    }

    impl Config {
        pub fn new(args: &[String]) -> Result<Config, &'static str> {
            if args.len() < 2 {
                return Err("Not enough arguments!");
            }

            let filename = args[1].clone();

            let extension = filename.split('.').last();

            if let Some(extension) = extension {
                let extension: Extension = match extension {
                    "bmp" => { Extension::BMP },
                    _ => { return Err("Unknown file extension") },
                };
                return Ok(Config { filename: PathBuf::from(Path::new(&filename)), extension });
            }
            
            Err("Could not process filename")
        }
    }
}



