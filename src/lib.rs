use std::{fs::File, str::FromStr};

use image::{
    imageops::{index_colors, BiLevel, ColorMap},
    DynamicImage, GenericImageView, ImageBuffer, Luma, Pixel, Rgb, RgbImage, Rgba, RgbaImage,
};
use indicatif::ProgressBar;
use num::complex::Complex;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    // lightest colours only
    Light,
    // darkest colours only
    Dark,
    // random,
    Random,
}

#[derive(Debug)]
pub struct ParseModeError {}

impl FromStr for Mode {
    type Err = ParseModeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.split_whitespace().collect::<String>().to_lowercase();
        if (s == "dark") | (s == "d") {
            Ok(Mode::Dark)
        } else if (s == "light") | (s == "l") {
            Ok(Mode::Light)
        } else if (s == "random") | (s == "r") {
            Ok(Mode::Random)
        } else {
            Err(ParseModeError {})
        }
    }
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match self {
            Mode::Dark => "dark".to_string(),
            Mode::Light => "light".to_string(),
            Mode::Random => "random".to_string(),
        }
    }
}

pub fn map_onto_whitespace(img: &DynamicImage, mode: &Mode) -> DynamicImage {
    println!("splitting image by luminance...");
    let (w, h) = img.dimensions();
    let rgb_img = img.to_rgb8();
    let img_luma = img.to_luma8();
    let cmap = BiLevel;
    let palletised = index_colors(&img_luma, &cmap);

    //map onto black / white

    /*
    let mapped = ImageBuffer::from_fn(w, h, |x, y| {
            let p = palletised.get_pixel(x, y);
            cmap.lookup(p.0[0] as usize)
            .expect("indexed color out-of-range")
    });
     */

    match mode {
        Mode::Light => {
            let colour_mapped = ImageBuffer::from_fn(w, h, |x, y| {
                // why doesnt this complete to (w*h ?)
                // bar.inc(1);

                let p = palletised.get_pixel(x, y);
                // match colour mapped pixel (0, 1) => (white, colour) from original image. Switch around to make it (0 or 1) => (colour or white)
                match p.0[0] {
                    0 => Rgb::from([255u8, 255u8, 255u8]),
                    1 => *rgb_img.get_pixel(x, y),
                    _ => panic!(),
                }
            });
            DynamicImage::ImageRgb8(colour_mapped)
        }

        Mode::Dark => {
            let colour_mapped = ImageBuffer::from_fn(w, h, |x, y| {
                // why doesnt this complete to (w*h ?)
                // bar.inc(1);

                let p = palletised.get_pixel(x, y);
                // match colour mapped pixel (0, 1) => (white, colour) from original image. Switch around to make it (0 or 1) => (colour or white)
                match p.0[0] {
                    0 => *rgb_img.get_pixel(x, y),
                    1 => Rgb::from([255u8, 255u8, 255u8]),
                    _ => panic!(),
                }
            });
            DynamicImage::ImageRgb8(colour_mapped)
        }

        _ => {
            let mapped = ImageBuffer::from_fn(w, h, |x, y| {
                let p = palletised.get_pixel(x, y);
                cmap.lookup(p.0[0] as usize)
                    .expect("indexed color out-of-range")
            });
            DynamicImage::ImageLuma8(mapped)
        }
    }

    //colour_mapped.save("output/bilevel.png");
}

pub fn pixelate(img: DynamicImage, output_width: u32) -> DynamicImage {
    println!("Pixelating...");
    let img = img.to_rgba8();

    let (width, height) = img.dimensions();
    println!("width = {},  output_width = {}", width, output_width);

    // size of square subsections to average pixels
    let sub_image_width = width / output_width;
    let sub_image_area = sub_image_width * sub_image_width;

    // calc height of output image
    let output_height = height / sub_image_width;
    let output_area = output_width * output_height;

    // crop to fit n equal large pixels

    //println!("Image Dimensions: {}x{}", width, height);
    //println!("SubImage size: {}x{}", sub_image_width, sub_image_width);
    //println!("Pixelated Image dimensions: {}x{}\n", output_width, output_height);

    // create new small image of sample size
    let mut output_img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::new(output_width, output_height);

    let bar = ProgressBar::new(output_area.into());

    for y in 0..output_height {
        for x in 0..output_width {
            // create subimage of new pixel area from original image
            let sub_img = img
                .view(
                    x * sub_image_width,
                    y * sub_image_width,
                    sub_image_width,
                    sub_image_width,
                )
                .to_image();

            let (mut r_total, mut g_total, mut b_total, mut a_total) = (0u32, 0u32, 0u32, 0u32);
            for pixel in sub_img.pixels() {
                if let [r, g, b, a] = pixel.channels() {
                    r_total += *r as u32;
                    g_total += *g as u32;
                    b_total += *b as u32;
                    a_total += *a as u32;
                }
            }

            let average_pixel = [
                (r_total / sub_image_area) as u8,
                (g_total / sub_image_area) as u8,
                (b_total / sub_image_area) as u8,
                (a_total / sub_image_area) as u8,
            ];
            //println!("avg rgb: {:?}", average_pixel);

            // paint average pixel of subimage into subimage location
            output_img.put_pixel(x, y, Rgba(average_pixel));

            bar.inc(1)
        }
    }

    //println!("{:?}", output_img.dimensions());
    DynamicImage::ImageRgba8(output_img)
}

pub fn create_background(
    foreground: ImageBuffer<Rgba<u8>, Vec<u8>>,
    colour: &BackgroundColour,
) -> DynamicImage {
    let mut composite: ImageBuffer<Rgba<u8>, Vec<u8>> = foreground;
    match *colour {
        BackgroundColour::Black => {
            for (_, _, pix) in composite.enumerate_pixels_mut() {
                if let [_, _, _, a] = pix.channels() {
                    // blend alhpa values
                    pix.blend(&Rgba([0, 0, 0, 255 - a]))
                }
            }
        }
        BackgroundColour::White => {
            for (_, _, pix) in composite.enumerate_pixels_mut() {
                if let [_, _, _, a] = pix.channels() {
                    // blend alhpa values
                    pix.blend(&Rgba([255, 255, 255, 255 - a]))
                }
            }
        }
        BackgroundColour::Custom((r, g, b)) => {
            for (_, _, pix) in composite.enumerate_pixels_mut() {
                if let [_, _, _, a] = pix.channels() {
                    // blend alhpa values
                    pix.blend(&Rgba([r, g, b, 255 - a]))
                }
            }
        }
    }

    // find a functional approach like this but actually works ?
    //let composite = foreground.pixels_mut().zip(background.pixels_mut()).map(|(pix_f, pix_b)| pix_f.blend(pix_b)).collect();
    DynamicImage::from(composite)
}

/*
fn step_universe(universe: Universe, (width, height): (u32, u32)) {
    for x in 0..10 {
        let universe = step(&universe.cells, &universe.image, (width, height));
        println!("saving image...");
        universe.image.save(format!("output/life/{}.png", x));
    }

}
*/

pub fn make_gif(slides: Vec<RgbImage>, speed: bool, file_path: &str) {
    println!("creating gif at {}.gif", file_path);

    let mut image = File::create(format!("{}.gif", file_path)).expect("couldn't save gif to path:");
    let (width, height) = slides[0].dimensions();
    let mut encoder = gif::Encoder::new(&mut image, width as u16, height as u16, &[]).unwrap();

    let bar = ProgressBar::new(slides.len() as u64);

    if speed {
        for img in slides {
            let pixels = img.into_raw();
            let frame = gif::Frame::from_rgb_speed(width as u16, height as u16, &*pixels, 10);

            // Write frame to file
            encoder.write_frame(&frame).unwrap();
            bar.inc(1);
        }
    } else {
        for img in slides {
            let pixels = img.into_raw();
            let frame = gif::Frame::from_rgb(width as u16, height as u16, &*pixels);

            // Write frame to file
            encoder.write_frame(&frame).unwrap();
            bar.inc(1);
        }
    }
}

pub fn split() {
    let (w, h) = (16, 16);
    // Create an image with a smooth horizontal gradient from black (0) to white (255).
    let grey = ImageBuffer::from_fn(w, h, |x, _y| -> Luma<u8> { [(255 * x / w) as u8].into() });
    // Mapping the gray image through the `BiLevel` filter should map gray pixels less than half
    // intensity (127) to black (0), and anything greater to white (255).
    let cmap = BiLevel;
    let palletized = index_colors(&grey, &cmap);
    let mapped = ImageBuffer::from_fn(w, h, |x, y| {
        let p = palletized.get_pixel(x, y);
        cmap.lookup(p.0[0] as usize)
            .expect("indexed color out-of-range")
    });
    // Create an black and white image of expected output.
    let bw = ImageBuffer::from_fn(w, h, |x, _y| -> Luma<u8> {
        if x <= (w / 2) {
            [0].into()
        } else {
            [255].into()
        }
    });
    assert_eq!(mapped, bw);

    //bw.save("output/bilevel.png");
}

pub fn ops() {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let img = image::open("images/salamence.png").unwrap();

    // The dimensions method returns the images width and height.
    println!("dimensions {:?}", img.dimensions());

    // The color method returns the image's `ColorType`.
    println!("{:?}", img.color());

    println!("blurring image...");
    let img = img.blur(5.0);

    // Write the contents of this image to the Writer in PNG format.
    img.save("output/test.png").unwrap();
}

pub fn make_image((width, height): (u32, u32)) -> RgbaImage {
    // Construct a new RGB ImageBuffer with the specified width and height.
    let mut img: RgbaImage = ImageBuffer::new(width, height);

    // Construct a new by repeated calls to the supplied closure.
    /*
    let mut img = ImageBuffer::from_fn(512, 512, |x, y| {
        if x % 2 == 0 {
            image::Luma([0u8])
        } else {
            image::Luma([255u8])
        }
    });  */

    // access pixel at coordinates (100, 100)
    // let pixel = img[(100, 100)];
    // or use 'get_pixel' method from 'genericImage' trait
    // let pixel = *img.get_pixel(100, 100);

    // put pixel at coordinate
    //img.put_pixel(100, 100, Rgba([255u8, 255u8, 255u8, 255u8]));

    // iterate over all pixels in the image.
    for (_x, _y, pixel) in img.enumerate_pixels_mut() {
        *pixel = image::Rgba([255, 255, 255, 255]);
    }
    img
}

pub fn julia() {
    let imgx = 800;
    let imgy = 800;

    let scalex = 3.0 / imgx as f32;
    let scaley = 3.0 / imgy as f32;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (0.3 * x as f32) as u8;
        let b = (0.3 * y as f32) as u8;
        *pixel = image::Rgb([r, 0, b]);
    }

    // A redundant loop to demonstrate reading image data
    for x in 0..imgx {
        for y in 0..imgy {
            let cx = y as f32 * scalex - 1.5;
            let cy = x as f32 * scaley - 1.5;

            let c = Complex::new(-0.4, 0.6);
            let mut z = Complex::new(cx, cy);

            let mut i = 0;
            while i < 255 && z.norm() <= 2.0 {
                z = z * z + c;
                i += 1;
            }

            let pixel = imgbuf.get_pixel_mut(x, y);
            let image::Rgb(data) = *pixel;
            *pixel = image::Rgb([data[0], i as u8, data[2]]);
        }
    }

    // Save the image as “fractal.png”, the format is deduced from the path
    imgbuf.save("output/fractal.png").unwrap();
}

pub fn cross() {
    let mut img = RgbImage::new(32, 32);

    for x in 15..=17 {
        for y in 8..24 {
            img.put_pixel(x, y, Rgb([255, 0, 0]));
            img.put_pixel(y, x, Rgb([255, 0, 0]));
        }
    }

    //img.save("output/test.png");
}

pub fn fit(width: u32, height: u32) -> Option<Vec<(u32, u32)>> {
    // find if you can evenly split image into squares
    // find common multiples of width and height
    let mut cm: Vec<(u32, u32)> = Vec::new();
    for x in 2..width {
        if width % x == 0 && height % x == 0 {
            cm.push((width / x, height / x));
        }
    }

    if cm.is_empty() {
        println!("no common multiples, should crop...");
        None
    } else {
        println!("can squash perfectly into dimensions {:?}", cm);
        Some(cm)
        //println!("popped {:?}", new_dimensions);
        //let (width, height) = new_dimensions;
    }
}

pub enum BackgroundColour {
    Black,
    White,
    Custom((u8, u8, u8)),
}

// todo: implement better errors
impl FromStr for BackgroundColour {
    type Err = String;
    // #0000FF;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.split_whitespace().collect::<String>().to_lowercase();
        match s.as_str() {
            "black" | "b" => Ok(Self::Black),
            "white" | "w" => Ok(Self::White),
            _ => {
                let colour = hex_rgb::convert_hexcode_to_rgb(s)?;
                let (r, g, b) = (colour.red, colour.green, colour.blue);
                Ok(Self::Custom((r, g, b)))
            }
        }
    }
}

impl Default for BackgroundColour {
    fn default() -> Self {
        BackgroundColour::Black
    }
}

pub fn background_for_slides(
    slides: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    colour: BackgroundColour,
) -> Vec<ImageBuffer<Rgb<u8>, Vec<u8>>> {
    let mut blended: Vec<RgbImage> = Vec::new();
    println!("applying background");
    let bar = ProgressBar::new(slides.len() as u64);
    for s in slides {
        let new_img = create_background(s, &colour).to_rgb8();
        blended.push(new_img);
        bar.inc(1);
    }
    blended
}
