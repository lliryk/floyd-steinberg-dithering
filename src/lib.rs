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

pub mod bit_map {
    use std::fmt::Debug;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;
    use thiserror::Error;

    pub struct BitMap {
        file_header: BitMapFileHeader,
        bin_header: BitMapInfoHeader,
        pixel_array: PixelArray,
    }

    #[derive(Error, Debug)]
    pub enum Issue {
        #[error("Error reading file {0}")]
        IoError(#[from] std::io::Error),

        #[error("Bad {0}, missing {1} bytes")]
        BadStruct(String, usize),
        
        #[error("Checksum Failed: found [{:x}, {:x}], expected [0x42, 0x4d]", .0, .1)]
        ChecksumFailure(u8, u8),

        #[error("Unsupported format: {:?}", .0)]
        UnsupportedCompression(Compression),
    }

    impl BitMap {
        pub fn new(filename: &Path) -> Result<BitMap, Issue> {
            let mut file = match File::open(filename) {
                Err(err) => return Err(Issue::IoError(err)),
                Ok(file) => file,
            };
            
            let mut buffer = Vec::new();
            if let Err(err) = file.read_to_end(&mut buffer) {
                return Err(Issue::IoError(err));
            }

            if buffer.len() < 14 {
                return Err(Issue::BadStruct("BitMapFileHeader".to_string(), 14 - buffer.len()));
            }

            let file_header: BitMapFileHeader = bincode::deserialize(&buffer[0..14]).unwrap();

            let identifier:[u8; 2] = [0x42, 0x4d]; 
            if !file_header.identifier.eq(&identifier) {
                return Err(Issue::ChecksumFailure(file_header.identifier[0], file_header.identifier[1]));
            }

            if buffer.len() < 18 {
                return Err(Issue::BadStruct("BitMapInfoHeader".to_string(), 18 - buffer.len()));
            }

            let bin_size: u32 = bincode::deserialize(&buffer[14..18]).unwrap();

            if buffer.len() < 14 + bin_size as usize {
                return Err(Issue::BadStruct("BitMapInfoHeader".to_string(), (14 + bin_size as usize) - buffer.len()));
            }

            let bin_header: BitMapInfoHeader = bincode::deserialize(&buffer[14..14 + bin_size as usize]).unwrap();

            match (bin_header.compression as i32).try_into() {
                Ok(Compression::RGB) => {},
                Ok(x) => return Err(Issue::UnsupportedCompression(x)),
                Err(_) => return Err(Issue::UnsupportedCompression(Compression::Unknown)),
            }

            let pixel_array_start = file_header.offset as usize;
            let pixel_array_end = pixel_array_start + bin_header.bitmap_size as usize;

            if (buffer.len() as usize) < pixel_array_end {
                return Err(Issue::BadStruct("PixelArray".to_string(), pixel_array_end - buffer.len()));
            }

            let pixel_array = PixelArray::new(
                bin_header.pixel_width as usize,
                bin_header.pixel_height.abs() as usize,
                &buffer[pixel_array_start..pixel_array_end],
                bin_header.pixel_height > 0
            );
           

            Ok(BitMap { file_header, bin_header, pixel_array })
        }

        pub fn save(&self, path: &Path) -> Result<usize, Issue> {
            let mut file = match File::create(path) {
                Err(why) => return Err(Issue::IoError(why)),
                Ok(file) => file,
            };

            let mut file_offset = 0;

            let buf = bincode::serialize(&self.file_header).unwrap();

            match file.write(&buf) {
                Err(why) => return Err(Issue::IoError(why)),
                Ok(size) => file_offset += size,
            }

            let buf = bincode::serialize(&self.bin_header).unwrap();

            match file.write(&buf) {
                Err(why) => return Err(Issue::IoError(why)),
                Ok(size) => file_offset += size,
            }

            if self.file_header.offset as usize > file_offset {
                let buf = vec![0; self.file_header.offset as usize - file_offset];
                match file.write(&buf) {
                    Err(why) => return Err(Issue::IoError(why)),
                    Ok(size) => file_offset += size,
                }
            }

            let mut buf = Vec::new();

            for pixel in &self.pixel_array.pixel_array(self.bin_header.pixel_height > 0) {
                buf.push(pixel.b);
                buf.push(pixel.g);
                buf.push(pixel.r);
            }

            while buf.len() % 4 != 0 {
                buf.push(0);
            }

            match file.write(&buf) {
                Err(why) => return Err(Issue::IoError(why)),
                Ok(size) => file_offset += size,
            }

            Ok(file_offset)
        }

        pub fn debug(&self) {
            println!("{:?}\n{:?}\n", self.file_header, self.bin_header);
        }

        
        pub fn dither_floydsteinberg(&mut self, nbits: i32) {
            // self.pixel_array.pixel_array = self.pixel_array.pixel_array.iter().map(|pixel| { pixel.convert_to_grayscale() }).collect();

            for y in 0..(self.bin_header.pixel_height - 1) {
                for x in 0..(self.bin_header.pixel_width - 1) {
                    let original = self.pixel_array.get_pixel(x, y);
                    let quantized = original.quantize_rgb_nbit(nbits);

                    let error:[i32; 3] = [
                        (original.r as i32 - quantized.r as i32), 
                        (original.g as i32 - quantized.g as i32), 
                        (original.b as i32 - quantized.b as i32)
                        ];
                    
                    self.pixel_array.set_pixel(x, y, quantized);

                    // Update the corresponding pixels surrounding the current one
                    let mut update_pixel = | offset: (i32, i32), error_bias: f32 | {
                        let x = x + offset.0;
                        let y = y + offset.1;
                        let pixel = self.pixel_array.get_pixel(x, y);
                        
                        let mut k = [pixel.r as i32, pixel.g as i32, pixel.b as i32];
                        k[0] += (error[0] as f32 * error_bias) as i32;
                        k[1] += (error[1] as f32 * error_bias) as i32;
                        k[2] += (error[2] as f32 * error_bias) as i32;

                        let pixel = Pixel { 
                            r: k[0].clamp(0, 255) as u8,
                            g: k[1].clamp(0, 255) as u8,
                            b: k[2].clamp(0, 255) as u8 
                        };
                        self.pixel_array.set_pixel(x, y, pixel);
                    };

                    update_pixel((1, 0), 7.0f32 / 16.0f32);
                    update_pixel((-1, 1), 3.0f32 / 16.0f32);
                    update_pixel((0, 1), 5.0f32 / 16.0f32);
                    update_pixel((1, 1), 1.0f32 / 16.0f32);
                }
            }
        }

    } 
      
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct BitMapFileHeader {
        identifier: [u8; 2],
        size: u32,
        resevered: [u8; 4], // Not used
        offset: u32,
    }

    #[derive(Debug)]
    pub enum Compression {
        RGB = 0,
        RLE8 = 1,
        RLE4 = 2,
        BITFIELDS = 3,
        JPEG = 4,
        PNG = 5,
        ALPHABITFIELDS = 6,
        CMYK = 11,
        CMYKRLE8 = 12,
        CMYKRLE4 = 13,

        Unknown = -1,
    }

    impl TryFrom<i32> for Compression {
        type Error = ();

        fn try_from(value: i32) -> Result<Self, Self::Error> {
            match value {
                v if v == Compression::RGB as i32 => Ok(Compression::RGB),
                v if v == Compression::RLE8 as i32 => Ok(Compression::RLE8),
                v if v == Compression::RLE4 as i32 => Ok(Compression::RLE4),
                v if v == Compression::BITFIELDS as i32 => Ok(Compression::BITFIELDS),
                v if v == Compression::JPEG as i32 => Ok(Compression::JPEG),
                v if v == Compression::PNG as i32 => Ok(Compression::PNG),
                v if v == Compression::ALPHABITFIELDS as i32 => Ok(Compression::ALPHABITFIELDS),
                v if v == Compression::CMYK as i32 => Ok(Compression::CMYK),
                v if v == Compression::CMYKRLE8 as i32 => Ok(Compression::CMYKRLE8),
                v if v == Compression::CMYKRLE4 as i32 => Ok(Compression::CMYKRLE4),
                _ => Err(()),
            }
        }
    } 

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct BitMapInfoHeader {
        size: u32, // Size of header in bytes (should be 40)
        pixel_width: i32,
        pixel_height: i32,
        color_planes: u16,
        color_depth: u16,
        compression: u32,
        bitmap_size: u32,
        width: i32,
        height: i32,
        colors: u32,
        important_colors: u32,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
    struct Pixel {
        r: u8,
        g: u8,
        b: u8,
    }

    impl Pixel {
        fn new(r: u8, g: u8, b: u8) -> Pixel {
            Pixel{r, g, b}
        }

        fn convert_to_grayscale(&self) -> Pixel {
            let grayscale = 0.2162f32 * self.r as f32 + 0.7152f32 * self.g as f32 + 0.0722f32 * self.b as f32;
            let grayscale = grayscale as u8;
            Pixel {r: grayscale, g: grayscale, b: grayscale}
        }

        fn quantize_grayscale_nbit(&self, nbit: i32) -> Pixel {

            let levels = ((1 << nbit) - 1) as f32;
            let color = (self.r as f32 / 255.0f32) * levels;
            let color = color.round();
            let color = color / levels * 255.0f32;
            let color = color.clamp(0.0f32, 255.0f32);

            let color = color as u8;

            Pixel { r: color, g: color, b: color }
        }

        fn quantize_rgb_nbit(&self, nbit: i32) -> Pixel {

            let levels = ((1 << nbit) - 1) as f32;
            // Red
            let red = (self.r as f32 / 255.0f32) * levels;
            let red = red.round();
            let red = red / levels * 255.0f32;
            let red = red.clamp(0.0f32, 255.0f32);

            let red = red as u8;

            // Green
            let green = (self.g as f32 / 255.0f32) * levels;
            let green = green.round();
            let green = green / levels * 255.0f32;
            let green = green.clamp(0.0f32, 255.0f32);

            let green = green as u8;

            // Blue
            let blue = (self.b as f32 / 255.0f32) * levels;
            let blue = blue.round();
            let blue = blue / levels * 255.0f32;
            let blue = blue.clamp(0.0f32, 255.0f32);

            let blue = blue as u8;


            Pixel { r: red, g: green, b: blue }
        }
    }
    struct PixelArray {
        pixel_array: Vec<Pixel>,
        width: usize,
        height: usize,
    }

    impl PixelArray {
        fn new(width: usize, height: usize, raw_pixels: &[u8], flip: bool) -> PixelArray {
             let mut pixel_array: Vec<Pixel> = Vec::new();
 
             // Look into how to make this not suck
             for chunk in raw_pixels.chunks(3) {
                 if chunk.len() != 3 {
                      break;
                    }
                 pixel_array.push(Pixel { r: (chunk[2]), g: (chunk[1]), b: (chunk[0]) });
             }

             if flip {
                let flipped = PixelArray::flip(width, height, &pixel_array);
                pixel_array = flipped;
             }

             PixelArray { pixel_array, width, height }
        }

        fn flip(width: usize, height: usize, pixel_array: &Vec<Pixel>) -> Vec<Pixel> {
            let mut flipped = vec![Pixel::new(0, 0, 0); pixel_array.len()];
            for y in (0..(height - 1) as usize).rev() {
                for x in 0..(width - 1) as usize {
                    let index = y * width + x; 
                    flipped[index] = pixel_array[index];
                }
            }

            flipped
        }

        fn get_pixel(&self, x: i32, y: i32) -> Pixel {
            if (x < 0 || x as usize > self.width - 1) || (y < 0 || y as usize > self.height - 1) {
                return Pixel { r: 0, g: 0 , b: 0 };
            }

            let x = x as usize;
            let y = y as usize;

            return self.pixel_array[y * self.width + x];
        }

        fn set_pixel(&mut self, x: i32, y: i32, pixel: Pixel) {
            if (x < 0 || x as usize > self.width - 1) || (y < 0 || y as usize > self.height - 1) {
                return;
            }
            
            let x = x as usize;
            let y = y as usize;

            self.pixel_array[y * self.width + x] = pixel;
        }

        fn pixel_array(&self, flip: bool) -> Vec<Pixel> {
            if flip {
                return PixelArray::flip(self.width, self.height, &self.pixel_array);
            }
            self.pixel_array.clone()
        }
    }
}


