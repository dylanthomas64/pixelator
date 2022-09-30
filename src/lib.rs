use image::{GenericImageView, RgbImage, ImageBuffer, Pixel, Rgb, DynamicImage, imageops::{BiLevel, index_colors, ColorMap}, Rgba};
use indicatif::ProgressBar;
use num::{complex::Complex, integer::Roots};








#[derive(Debug)]
pub enum Mode {
    // lightest colours only 
    Light,
    // darkest colours only
    Dark,
    // monochrome
    Monochrome,
}

pub fn map_onto_whitespace(img: &DynamicImage, mode: &Mode) -> DynamicImage {
    println!("splitting image by luminance...");
    let (w, h) = img.dimensions();
    let img_area = w*h;
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

    // map onto colour / white
    println!("drawing new image...");
    let bar = ProgressBar::new((w*h) as u64);
    
    let output_img = match mode {
        Mode::Light => {
            let colour_mapped = ImageBuffer::from_fn(w, h,  |x, y| { 
                // why doesnt this complete to (w*h ?)
                // bar.inc(1);
        
                let p = palletised.get_pixel(x, y);
                // match colour mapped pixel (0, 1) => (white, colour) from original image. Switch around to make it (0 or 1) => (colour or white)
                match p.0[0] {
                    0 => {Rgb::from([255u8, 255u8, 255u8])},
                    1 => {*rgb_img.get_pixel(x, y)},
                    _ => panic!()
                }
            });
            DynamicImage::ImageRgb8(colour_mapped)
        },

        Mode::Dark => {
            let colour_mapped = ImageBuffer::from_fn(w, h,  |x, y| { 
                // why doesnt this complete to (w*h ?)
                // bar.inc(1);
        
                let p = palletised.get_pixel(x, y);
                // match colour mapped pixel (0, 1) => (white, colour) from original image. Switch around to make it (0 or 1) => (colour or white)
                match p.0[0] {
                    0 => {*rgb_img.get_pixel(x, y)},
                    1 => {Rgb::from([255u8, 255u8, 255u8])},
                    _ => panic!()
                }
            });
            DynamicImage::ImageRgb8(colour_mapped)

        },

        _ => {
            let mapped = ImageBuffer::from_fn(w, h, |x, y| {
                let p = palletised.get_pixel(x, y);
                cmap.lookup(p.0[0] as usize)
                    .expect("indexed color out-of-range")
            });
            DynamicImage::ImageLuma8(mapped)
        },
    };

    
    output_img
    
    //colour_mapped.save("output/bilevel.png");

}


pub fn pixelate(img: DynamicImage, output_width: u32) -> DynamicImage {
    println!("Pixelating...");
    let colour = img.color();
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
    let mut output_img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(output_width, output_height);

    let bar = ProgressBar::new(output_area.into());

    for y in 0..output_height {
        for x in 0..output_width {    
            // create subimage of new pixel area from original image
            let sub_img = img.view(x*sub_image_width, y*sub_image_width, sub_image_width, sub_image_width).to_image();
    
            
            let (mut R, mut G, mut B, mut A) = (0u32, 0u32, 0u32, 0u32);
            for pixel in sub_img.pixels() {
                if let [r, g, b, a] = pixel.channels() {
                    R += *r as u32;
                    G += *g as u32;
                    B += *b as u32;
                    A += *a as u32;
                }
            }

            let average_pixel = [(R/sub_image_area) as u8, (G/sub_image_area) as u8, (B/sub_image_area) as u8,  (A/sub_image_area) as u8];
            //println!("avg rgb: {:?}", average_pixel);
    
    
            // paint average pixel of subimage into subimage location
            output_img.put_pixel(x, y, Rgba(average_pixel));


            bar.inc(1)

        }
    
    }
    


    //println!("{:?}", output_img.dimensions());
    return DynamicImage::ImageRgba8(output_img)


}


pub fn fit(width: u32, height: u32) -> Option<Vec<(u32, u32)>> {
    // find if you can evenly split image into squares
    // find common multiples of width and height
    let mut cm: Vec<(u32, u32)> = Vec::new();
    for x in 2..width {
        if width % x == 0 {
            if height % x == 0 {
                cm.push((width/x, height/x));
            }
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