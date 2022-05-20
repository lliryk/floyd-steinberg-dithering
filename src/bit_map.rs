use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use thiserror::Error;
use crate::pixels::{Pallete, PixelArray};

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

    
    pub fn dither_floydsteinberg(&mut self, pallete: &Pallete, nbits: i32) {
        self.pixel_array.dither_floydsteinberg(pallete, nbits);
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
