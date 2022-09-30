use std::fmt::Display;
use std::fs::{File, self};
use std::str::FromStr;
use std::time::Duration;

use image::{RgbaImage, Rgba};
use image::{GenericImageView, RgbImage, ImageBuffer, Pixel, Rgb, DynamicImage, Luma};
use image::imageops::colorops::{index_colors, BiLevel, ColorMap};

use gif::{Decoder, Encoder, Frame};
use image::{ImageDecoder, AnimationDecoder};

use pixelator::{pixelate, map_onto_whitespace, Mode, fit};
use indicatif::ProgressBar;
use num::{complex::Complex, integer::Roots};

mod conway;

use conway::{map_onto_cells};

use crate::conway::{neighbors, neighbors_coords, step, create_next_image, Universe, CellState};

use clap::Parser;

use rand::Rng;

#[derive(Parser, Debug)]
#[command(author, version, about= "does stuff", long_about = None)]
struct Args {
    #[arg(short, long)]
    /// path to image
    path: String,

    #[arg(short, long)]
    /// output directory. CWD if none given.
    output_directory: Option<String>,

    #[arg(short, long, default_value_t=100)]
    ///number of steps in the game of life
    generations: u64,

    /// 0-255, how much the colours decay each step. Deducts from alpha values each step.
    #[arg(short, long, default_value_t=32)]
    decay: u8,

    #[arg(short, long)]
    /// select "dark" or "light" mode for colour mapping options. Default = "random"
    mode: Option<String>,

    /// set true for faster lossy gif rendering
    #[arg(short, long, default_value_t=false)]
    speed: bool,

    /// output pixel width dimension using pixelation effect. Default is original image size.
    /// smaller picture sizes result in exponentially faster renders.
    #[arg(short, long)]
    width: Option<u32>,

}


fn create_directory(path: &str) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

fn main() {
    let args = Args::parse();
    println!("path : {}", args.path);

    let imagePath = args.path;

    

    let generations = args.generations;
    let decay = args.decay;

    let mode = match args.mode {
        Some(modeString) => Mode::from_str(&modeString).expect("not a valid mode"),
        // default = random
        None => Mode::Random,
    };
    
    // speedy gif making is lossy
    let speed = args.speed;
    // let reverse = true;


    let s = imagePath.split(['/', '.']).collect::<Vec<&str>>();
    let fileName = s[s.len()-2];
    //println!("{:?}", fileName);

    //determine output width for file name
    let output_width = match args.width {
        Some(width) => width.to_string(),
        none => "OG".to_string(),
    };

    let newFileName = format!("{}x{}_{}_{}_{}", fileName, output_width, mode.to_string(), generations, decay);

    // if output path specified then create if it doesn't already exist
    let output_path = if let Some(path) = args.output_directory {
        create_directory(&path).expect("couldn't create path");
        path + &newFileName
    } else {
        newFileName
    };

    



    println!("opening image...");
    let mut img = image::open(imagePath).expect("failed to load image");
    
    if let Some(width) = args.width {
        img = pixelate(img, width);
    }

    /* 
    let img = match args.width {
        Some(width) => pixelate(img, width),
        none => img
    } */
    

    let slides = begin_life(img, generations, decay, &mode);
    //let slides_smooth = begin_life(img, generations, 2, &mode);

 


    let mut blended: Vec<RgbImage> = Vec::new();
    println!("drawing backdrop");
    let bar = ProgressBar::new(slides.len() as u64);
    for s in slides {
        let new_img = create_background(s).to_rgb8();
        blended.push(new_img);
        bar.inc(1);
    }

    gif(blended, speed, &output_path);
    

}


fn create_background(foreground: ImageBuffer<Rgba<u8>, Vec<u8>>) -> DynamicImage {
    let (width, height) = foreground.dimensions();

    let mut composite: ImageBuffer<Rgba<u8>, Vec<u8>> = foreground;

    for (x, y, pix) in composite.enumerate_pixels_mut() {
        if let [r, g, b, a] = pix.channels() {
             // blend alhpa values
        pix.blend(&Rgba([0, 0, 0, 255-a]))
        }
       
    }

    // find a functional approach like this but actually works ?
    //let composite = foreground.pixels_mut().zip(background.pixels_mut()).map(|(pix_f, pix_b)| pix_f.blend(pix_b)).collect();
    DynamicImage::from(composite)
}

fn begin_life(img: DynamicImage, generations: u64, decay: u8, mode: &Mode) -> Vec<RgbaImage> {
    /// set decay 0-255, make equal to 2^n for smooth results. 32 is about right to witness pulsing
    /// light mode creates life on lightest pixels, dark mode creates life on darkest pixels

    println!("preparing first generation");

    let mut slides: Vec<RgbaImage> = Vec::new();
    
    let (width, height) = img.dimensions();
    let area = width * height;

    let mapped = map_onto_whitespace(&img, &mode);
    mapped.save("output/mapped.png");
    
    let cells = match mode {
        Mode::Random => {
            let mut cells = vec!(vec!(CellState::Dead; width as usize); height as usize);
            //create random life
            let mut rng = rand::thread_rng();
            for _n in 0..area {
                let x = rng.gen_range(0..width);
                let y = rng.gen_range(0..height);
                cells[y as usize][x as usize] = CellState::Alive;
            }
            cells
        },
        _ => {
            let cells = map_onto_cells(&img, &mode);
            cells
        }
    };

    let mut universe = Universe {
        cells: cells,
        image: img.into_rgba8(),
    };
    

    // start vector with a few original pixelated versions
    for x in 0..5 {
        slides.push(universe.image.clone());
    }

    let bar = ProgressBar::new(generations);

    println!("starting first generation...");
        for x in 0..generations {
            // try do it without clone...
            universe = step(universe.cells, &universe.image, (width, height), decay);
            let slide = universe.image.clone();
            slides.push(slide);
            bar.inc(1);

        }
    //step_universe(universe, (width, height))
    slides

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

fn gif(slides: Vec<RgbImage>, speed: bool, file_path: &str) {
    
        println!("creating gif...");
  
        let mut image = File::create(format!("{}.gif", file_path)).expect("couldn't save gif to path");
        let (width, height) = slides[0].dimensions();
        let mut encoder = gif::Encoder::new(&mut image, width as u16, height as u16, &[]).unwrap();
        let mut image_copy: RgbaImage;

        //let reverse: Vec<RgbImage> = slides.as_slice().reverse().collect();
        //let slides_with_reverse = slides.push(reverse);

        let bar = ProgressBar::new(slides.len() as u64);

        if speed {
            for img in slides {

                let mut pixels = img.into_raw();
                let frame = gif::Frame::from_rgb_speed(width as u16, height as u16, &mut *pixels, 10);
                
                // Write frame to file
                encoder.write_frame(&frame).unwrap();
                bar.inc(1);
            }

        } else {
            for img in slides {

                let mut pixels = img.into_raw();
                let frame = gif::Frame::from_rgb(width as u16, height as u16, &mut *pixels);
                
                // Write frame to file
                encoder.write_frame(&frame).unwrap();
                bar.inc(1);
            }
        }

        
    println!("saving gif...");
}













fn split() {


let (w, h) = (16, 16);
// Create an image with a smooth horizontal gradient from black (0) to white (255).
let gray = ImageBuffer::from_fn(w, h, |x, y| -> Luma<u8> { [(255 * x / w) as u8].into() });
// Mapping the gray image through the `BiLevel` filter should map gray pixels less than half
// intensity (127) to black (0), and anything greater to white (255).
let cmap = BiLevel;
let palletized = index_colors(&gray, &cmap);
let mapped = ImageBuffer::from_fn(w, h, |x, y| {
    let p = palletized.get_pixel(x, y);
    cmap.lookup(p.0[0] as usize)
        .expect("indexed color out-of-range")
});
// Create an black and white image of expected output.
let bw = ImageBuffer::from_fn(w, h, |x, y| -> Luma<u8> {
    if x <= (w / 2) {
        [0].into()
    } else {
        [255].into()
    }
});
assert_eq!(mapped, bw);

bw.save("output/bilevel.png");
}




fn ops() {
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

fn make_image((width, height): (u32, u32)) -> RgbaImage {
    // Construct a new RGB ImageBuffer with the specified width and height.
    let mut img: RgbaImage = ImageBuffer::new(width, width);

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
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = image::Rgba([255, 255, 255, 255]);
    }
    img
}

fn julia() {
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

fn cross() {

    let mut img = RgbImage::new(32, 32);

    for x in 15..=17 {
        for y in 8..24 {
            img.put_pixel(x, y, Rgb([255, 0, 0]));
            img.put_pixel(y, x, Rgb([255, 0, 0]));
        }
    }

    img.save("output/test.png");
}

fn convert() {
    let rgba = image::open("images/cacteinstein.jpg").unwrap().into_rgba8();
    let gray = image::DynamicImage::ImageRgba8(rgba).into_luma8();
    gray.save("output/test.png");
}

fn from_raw() {

    //let buffer: &[u8] = unimplemented!(); // Generate the image data

    // Save the buffer as "image.png"
    //image::save_buffer("image.png", buffer, 800, 600, image::ColorType::Rgb8).unwrap()
}