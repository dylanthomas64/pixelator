use image::{self, RgbImage, ImageBuffer, Rgb, DynamicImage, imageops::{index_colors, BiLevel}, GenericImageView, Pixel};
use indicatif::ProgressBar;
use pixelator::Mode;




#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellState {
    Alive,
    Dead,
}

// create new blank rgb image (dark or light)
// each cell surviving cell stays the same
// each new birth combines colours of surrounding cells
// for each new cell put pixel of new rgb value into equivilant image coordinates


pub fn create_next_image((width, height): (u32, u32)) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let img: RgbImage = ImageBuffer::from_fn(width, height, |x, y| Rgb([255u8;3]) );
    img
}

#[derive(Clone)]
pub struct Universe {
    pub cells: Vec<Vec<CellState>>,
    pub image: ImageBuffer<Rgb<u8>, Vec<u8>>,
}


pub fn step(frame: Vec<Vec<CellState>>, img: ImageBuffer<Rgb<u8>, Vec<u8>>, (width, height): (u32, u32)) -> Universe {
    let mut next_img = create_next_image((width, height));
    let mut next_frame = vec![vec!(CellState::Dead; width as usize); height as usize];
    for (y, row) in &mut frame.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let n = neighbors((x as i16, y as i16), &frame);
            
            match cell {
                &CellState::Alive if (n == 2) | (n == 3) => {
                    let pix = img.get_pixel(x as u32, y as u32);
                    //println!("({},{}) survives!", x, y);
                    next_frame[y][x] = CellState::Alive;
                    next_img.put_pixel(x as u32, y as u32, *pix)

                }
                &CellState::Dead if n == 3 => {
                    //println!("({},{}) begins life!", x, y);
                    let coords = neighbors_coords((x as i16, y as i16), &frame, (width, height));
                    let (mut R, mut G, mut B) = (0u32, 0u32, 0u32);
            
                    for coord in &coords {
                        if let [r, g, b] = img.get_pixel(coord.0, coord.1).channels() {
                            R += *r as u32;
                            G += *g as u32;
                            B += *b as u32;
                        } else {
                            panic!()
                        }
 
                    }
                    let avg_pix = Rgb([(R/4) as u8, (G/4) as u8, (B/4) as u8]);

                    next_img.put_pixel(x as u32, y as u32, avg_pix);
                    next_frame[y][x] = CellState::Alive;




                },
    
                _ => {  
                    // do nothing, all die out as blank image used each time
                },
            }
        }
    }
    //return cells vec and new image
    Universe {
        cells: next_frame,
        image: next_img,
    }
}


pub fn neighbors((col, row): (i16, i16), cells: &Vec<Vec<CellState>>) -> u8 {

    // costly ?? vs inputing (w, h) as a parameter
    let (height, width) = (cells.len(), cells[0].len());
    let mut total = 0i16;
    // make sure self wont be included in total
    if cells[row as usize][col as usize] == CellState::Alive {
        total -= 1;
    }
    for y in row - 1..=row + 1 {
        for x in col - 1..=col + 1 {
            if (x < 0) | (y < 0) | (x >= width as i16) | (y >= height as i16) {
                //println!("nope @ ({},{})", x, y);
            } else {
                let cell = cells[y as usize][x as usize];
                match cell {
                    CellState::Alive => {
                        total += 1;
                        //println!("yep @ ({},{})", x, y)
                    }
                    CellState::Dead => {
                        //println!("dead cell @ ({},{})", x, y)
                    }
                }
            }
        }
    }
    total as u8
}



// make next generation live cell a blend of self + surrounding cells
pub fn neighbors_coords((col, row): (i16, i16), cells: &Vec<Vec<CellState>>, (width, height): (u32, u32)) -> Vec<(u32, u32)> {
    
    let mut coords: Vec<(u32, u32)> = Vec::new();

    for y in row - 1..=row + 1 {
        for x in col - 1..=col + 1 {
            if (x < 0) | (y < 0) | (x >= width as i16) | (y >= height as i16) {
                //println!("nope @ ({},{})", x, y);
            } else {
                let cell = cells[y as usize][x as usize];
                match cell {
                    CellState::Alive => {
                        coords.push((x as u32, y as u32));
                        //println!("yep @ ({},{})", x, y)
                    }
                    CellState::Dead => {
                        //println!("dead cell @ ({},{})", x, y)
                    }
                }
            }
        }
    }
    coords
}













pub fn map_onto_cells(img: &DynamicImage, mode: &Mode) -> Vec<Vec<CellState>> {
    println!("splitting image by luminance...");
    let (w, h) = img.dimensions();
    let img_area = w*h;
    let rgb_img = img.to_rgb8();
    let img_luma = img.to_luma8();
    let cmap = BiLevel;
    let mut palletised = index_colors(&img_luma, &cmap);

    // create cell frame of dead cells
    // create h vectors or w length. index into using vec[y][x]
    let mut cells = vec!(vec!(CellState::Dead; w as usize); h as usize);

    println!("drawing new image...");
    let bar = ProgressBar::new((w*h) as u64);
    
    let output_cells = match mode {
        Mode::Light => {
        
            for (x, y, pix) in palletised.enumerate_pixels_mut() {
                let (x, y) = (x as usize, y as usize);
                match pix.0[0] {
                    0 => {},
                    1 => {cells[y][x] = CellState::Alive},
                    _ => panic!()
                }

            }
            cells

        },

        Mode::Dark | Mode::Monochrome => {

            for (x, y, pix) in palletised.enumerate_pixels_mut() {
                let (x, y) = (x as usize, y as usize);
                match pix.0[0] {
                    0 => {cells[y][x] = CellState::Alive},
                    1 => {},
                    _ => panic!()
                }

            }
            cells
        }
        
        _ => { panic!() },
    };

    output_cells
    
    //colour_mapped.save("output/bilevel.png");

}

pub fn make_image() -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // Construct a new RGB ImageBuffer with the specified width and height.
    let mut img: RgbImage = ImageBuffer::new(8, 8);

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
    //img.put_pixel(100, 100, Rgb([255u8, 255u8, 255u8]));

    // iterate over all pixels in the image.
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        if x % 3 == 0 {
            *pixel = image::Rgb([0, 0, 0]);
        } else {
            *pixel = image::Rgb([0, 255, 0]);
        }
        
    }

    img


}

pub fn make_image_from_vec(vec: Vec<Rgb<u8>>, (width, height): (u32, u32)) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // Construct a new RGB ImageBuffer with the specified width and height.
    let mut img: RgbImage = ImageBuffer::new(width, height);

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
    //img.put_pixel(100, 100, Rgb([255u8, 255u8, 255u8]));

    // iterate over all pixels in the image.
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        if x % 3 == 0 {
            *pixel = image::Rgb([255, 255, 255]);
        } else {
            *pixel = image::Rgb([0, 255, 0]);
        }
        
    }

    img


}