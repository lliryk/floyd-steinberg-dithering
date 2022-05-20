pub mod pixels;
pub mod bit_map;

pub mod config {
    use clap::Parser;
    use thiserror::Error;
    use std::str::FromStr;

    use crate::pixels::Pallete;

    #[derive(Debug)]
    pub enum Extension {
        BMP,
    }

    #[derive(Error, Debug)]
    pub enum Issue {
        #[error("Unknown Extension: {0}")]
        UnknownExtension(String),

        #[error("Could not process extension")]
        InvalidExtension,
    }

    impl FromStr for Extension {
        type Err = Issue;
        
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let extension = s.split('.').last();
            
            if let Some(extension) = extension {
                let extension: Extension = match extension {
                    "bmp" => { Extension::BMP },
                    _ => { return Err(Issue::UnknownExtension(extension.to_string())) },
                };
                return Ok(extension);
            }

            Err(Issue::InvalidExtension)
        }
    }

    /// Basic implementation of the floyd-steinberg dithering algorithm
    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about = None)]
    pub struct Config {
        /// Path to image to process
        #[clap(short, long)]
        pub filename: std::path::PathBuf,

        /// HTML basic colors seperated by commas: "red, green, blue"
        #[clap(short, long)]
        pub color_string: String,
       
        /// Bits per color
        #[clap(short, long)]
        pub bits: u8,

        /// Path of output file
        #[clap(short, long)]
        pub output: std::path::PathBuf,
    }

    impl Config {
        pub fn ext(&self) -> Result<Extension, Issue> {
            if let Some(ext) = self.filename.extension() {
                if let Some(ext) = ext.to_str() {
                    match Extension::from_str(ext) {
                        Ok(ext) => { return Ok(ext) },
                        Err(err) => { return Err(err) }
                    }
                }
            } 
            Err(Issue::InvalidExtension)     
        }

        pub fn pallete(&self) -> Pallete {
            let colors: Vec<String> = self.color_string.split(',')
            .map(str::trim).map(str::to_ascii_lowercase).collect();

            let color_ref: Vec<&str> = colors.iter().map(|x| x.as_ref()).collect();

            Pallete::new(&color_ref)
        }
    }
}



