use std::time::Duration;

use image::Frame;
use image::{GenericImageView, RgbImage, ImageBuffer, Pixel, Rgb, DynamicImage, Luma};
use image::imageops::colorops::{index_colors, BiLevel, ColorMap};

use pixelator::{pixelate, map_onto_whitespace, Mode};
use indicatif::ProgressBar;
use num::{complex::Complex, integer::Roots};

mod conway;

use conway::{map_onto_cells};

use crate::conway::{neighbors, neighbors_coords, step, create_next_image, Universe};



fn main() {

    




    println!("loading image...");
    let img = image::open("images/fungus.jpg").unwrap();
    let img = pixelate(img, 100);
    img.save("output/pixelated.png");


    let (width, height) = img.dimensions();

    let mode = Mode::Dark;

    let mapped = map_onto_whitespace(&img, &mode);
    mapped.save("output/mapped.png");
    
    let cells = map_onto_cells(&img, &mode);
    let mut universe = Universe {
        cells: cells,
        image: img.into_rgb8(),
    };

    //step_universe(universe, (width, height))
    for x in 0..50 {
        universe = step(universe.cells, universe.image, (width, height));
        println!("saving image...");
        universe.image.save(format!("output/life/{}.png", x));
    }

    


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

fn make_image() {
    // Construct a new RGB ImageBuffer with the specified width and height.
    let mut img: RgbImage = ImageBuffer::new(512, 512);

    // Construct a new by repeated calls to the supplied closure.
    /*
    let mut img = ImageBuffer::from_fn(512, 512, |x, y| {
        if x % 2 == 0 {
            image::Luma([0u8])
        } else {
            image::Luma([255u8])
        }
    });  */


    let (width, height) = img.dimensions();

    // access pixel at coordinates (100, 100)
    // let pixel = img[(100, 100)];
    // or use 'get_pixel' method from 'genericImage' trait
    // let pixel = *img.get_pixel(100, 100);

    // put pixel at coordinate
    img.put_pixel(100, 100, Rgb([255u8, 255u8, 255u8]));

    // iterate over all pixels in the image.
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        if x % 3 == 0 {
            *pixel = image::Rgb([255, 255, 255]);
        } else {
            *pixel = image::Rgb([255, 0, 0]);
        }
        
    }



    img.save("output/test.png").unwrap();
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