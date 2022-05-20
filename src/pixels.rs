use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Pixel {
    fn new(r: u8, g: u8, b: u8) -> Pixel {
        Pixel{r, g, b}
    }

    fn is_monochrome(&self) -> bool {
        (self.r == 0 && self.g == 0 && self.b == 0) ||
        (self.r == 255 && self.g == 255 && self.b == 255)
    }

    fn scale(&self, scale: Pixel) -> Pixel {
        let r = self.r as f32 * (scale.r as f32 / 255.0f32);
        let g = self.g as f32 * (scale.g as f32 / 255.0f32);
        let b = self.b as f32 * (scale.b as f32 / 255.0f32);

        // Should we clamp here?
        let r = r as u8;
        let g = g as u8;
        let b = b as u8;
        Pixel::new(r, g, b)
    }
    
    #[allow(dead_code)]
    fn convert_to_grayscale(&self) -> Pixel {
        let grayscale = 0.2162f32 * self.r as f32 + 0.7152f32 * self.g as f32 + 0.0722f32 * self.b as f32;
        let grayscale = grayscale as u8;
        Pixel {r: grayscale, g: grayscale, b: grayscale}
    }

    #[allow(dead_code)]
    fn quantize_grayscale_nbit(&self, nbit: i32) -> Pixel {

        let levels = ((1 << nbit) - 1) as f32;
        let color = (self.r as f32 / 255.0f32) * levels;
        let color = color.round();
        let color = color / levels * 255.0f32;
        let color = color.clamp(0.0f32, 255.0f32);

        let color = color as u8;

        Pixel { r: color, g: color, b: color }
    }

    fn quantize_rgb_nbit(&self, pallete: &Pallete, nbit: i32) -> Pixel {

        let quantised_color = self.quantize_rgb_pallete(&pallete.colors);

        let color;

        if quantised_color.is_monochrome() {
            color = self.convert_to_grayscale();
        } else {
            let scale = self.convert_to_grayscale();

            color = quantised_color.scale(scale);
        }

        let levels = ((1 << nbit) - 1) as f32;
        
        // Red
        let red = (color.r as f32 / 255.0f32) * levels;
        let red = red.round();
        let red = red / levels * 255.0f32;
        let red = red.clamp(0.0f32, 255.0f32);

        let red = red as u8;

        // Green
        let green = (color.g as f32 / 255.0f32) * levels;
        let green = green.round();
        let green = green / levels * 255.0f32;
        let green = green.clamp(0.0f32, 255.0f32);

        let green = green as u8;

        // Blue
        let blue = (color.b as f32 / 255.0f32) * levels;
        let blue = blue.round();
        let blue = blue / levels * 255.0f32;
        let blue = blue.clamp(0.0f32, 255.0f32);

        let blue = blue as u8;


        Pixel { r: red, g: green, b: blue }
    }

    fn quantize_rgb_pallete(&self, pallete: &[Pixel]) -> Pixel {
        let mut closest_distance = f32::INFINITY;
        let mut closest_index = 0;

        for (index, color) in pallete.iter().enumerate() {
            let distance = f32::sqrt(
                f32::powi(color.r as f32 - self.r as f32, 2) +
                f32::powi(color.g as f32 - self.g as f32, 2) + 
                f32::powi(color.b as f32 - self.b as f32, 2)
            );

            if distance < closest_distance {
                closest_distance = distance;
                closest_index = index;
            }
        }

        pallete[closest_index]
    }

}

pub struct Pallete {
    colors: Vec<Pixel>,
}

impl Pallete {
    pub fn new(color_names: &[&str]) -> Pallete {

        // Don't know how to build this statically
        let mut color_map = HashMap::new();

        color_map.insert("red", Pixel::new(255, 0, 0));
        color_map.insert("green", Pixel::new(0, 255, 0));
        color_map.insert("blue", Pixel::new(0, 0, 255));
        color_map.insert("white", Pixel::new(255, 255, 255));
        color_map.insert("black", Pixel::new(0, 0, 0));

        let mut colors = Vec::new();
        for color in color_names {
            if let Some(color_pixel) = color_map.get(color) {
                colors.push(color_pixel.clone());
            }
        }

        Pallete { colors }
    }
}

pub struct PixelArray {
    pixel_array: Vec<Pixel>,
    width: usize,
    height: usize,
}

impl PixelArray {
    pub fn new(width: usize, height: usize, raw_pixels: &[u8], flip: bool) -> PixelArray {
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

    pub fn pixel_array(&self, flip: bool) -> Vec<Pixel> {
        if flip {
            return PixelArray::flip(self.width, self.height, &self.pixel_array);
        }
        self.pixel_array.clone()
    }

    pub fn dither_floydsteinberg(&mut self, pallete: &Pallete, nbits: i32) {
        let height: i32 = self.height.try_into().unwrap();
        let width: i32 = self.width.try_into().unwrap();
        for y in 0..(height - 1) {
            for x in 0..(width - 1) {
                let original = self.get_pixel(x, y);
                let quantized = original.quantize_rgb_nbit(pallete, nbits);

                let error:[i32; 3] = [
                    (original.r as i32 - quantized.r as i32), 
                    (original.g as i32 - quantized.g as i32), 
                    (original.b as i32 - quantized.b as i32)
                    ];
                
                self.set_pixel(x, y, quantized);

                // Update the corresponding pixels surrounding the current one
                let mut update_pixel = | offset: (i32, i32), error_bias: f32 | {
                    let x = x + offset.0;
                    let y = y + offset.1;
                    let pixel = self.get_pixel(x, y);
                    
                    let mut k = [pixel.r as i32, pixel.g as i32, pixel.b as i32];
                    k[0] += (error[0] as f32 * error_bias) as i32;
                    k[1] += (error[1] as f32 * error_bias) as i32;
                    k[2] += (error[2] as f32 * error_bias) as i32;

                    let pixel = Pixel { 
                        r: k[0].clamp(0, 255) as u8,
                        g: k[1].clamp(0, 255) as u8,
                        b: k[2].clamp(0, 255) as u8 
                    };
                    self.set_pixel(x, y, pixel);
                };

                update_pixel((1, 0), 7.0f32 / 16.0f32);
                update_pixel((-1, 1), 3.0f32 / 16.0f32);
                update_pixel((0, 1), 5.0f32 / 16.0f32);
                update_pixel((1, 1), 1.0f32 / 16.0f32);
            }
        }
    }
}
