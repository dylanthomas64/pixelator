use image::{
    self,
    imageops::{index_colors, BiLevel},
    DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgba, RgbaImage,
};

use crate::Mode;
use indicatif::ProgressBar;
use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]

pub enum CellState {
    Alive,
    Dead,
}

// create new blank rgb image (dark or light)
// each cell surviving cell stays the same
// each new birth combines colours of surrounding cells
// for each new cell put pixel of new rgb value into equivilant image coordinates

#[derive(Clone)]
pub struct Universe {
    pub cells: Vec<Vec<CellState>>,
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

pub fn begin_life(img: DynamicImage, generations: u64, decay: u8, mode: &Mode) -> Vec<RgbaImage> {
    // set decay 0-255, make equal to 2^n for smooth results. 32 is about right to witness pulsing
    // light mode creates life on lightest pixels, dark mode creates life on darkest pixels

    let mut slides: Vec<RgbaImage> = Vec::new();

    let (width, height) = img.dimensions();
    let area = width * height;

    //let mapped = map_onto_whitespace(&img, &mode);
    //mapped.save("output/mapped.png");

    let cells = match mode {
        Mode::Random => {
            let mut cells = vec![vec!(CellState::Dead; width as usize); height as usize];
            //create random life
            let mut rng = rand::thread_rng();
            for _n in 0..area {
                let x = rng.gen_range(0..width);
                let y = rng.gen_range(0..height);
                cells[y as usize][x as usize] = CellState::Alive;
            }
            cells
        }
        _ => map_onto_cells(&img, mode),
    };

    let mut universe = Universe {
        cells,
        image: img.into_rgba8(),
    };

    // start vector with a few original pixelated versions
    for _ in 0..5 {
        slides.push(universe.image.clone());
    }

    let bar = ProgressBar::new(generations);

    println!("starting the game of life...");
    for _ in 0..generations {
        // try do it without clone...
        universe = step(universe.cells, &universe.image, (width, height), decay);
        let slide = universe.image.clone();
        slides.push(slide);
        bar.inc(1);
    }
    //step_universe(universe, (width, height))
    slides
}

pub fn step(
    frame: Vec<Vec<CellState>>,
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    (width, height): (u32, u32),
    alpha_decay_per_step: u8,
) -> Universe {
    let mut next_img = img.clone();
    let mut next_frame = vec![vec!(CellState::Dead; width as usize); height as usize];

    for (y, row) in &mut frame.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let n = neighbors((x as i16, y as i16), &frame);

            match *cell {
                // @ 2..=3?
                CellState::Alive if (n == 2) | (n == 3) => {
                    next_frame[y][x] = CellState::Alive;
                }

                CellState::Dead if n == 3 => {
                    let coords = neighbors_coords((x as i16, y as i16), &frame, (width, height));
                    let mut blended_pixel = Rgba([0; 4]);
                    for coord in &coords {
                        blended_pixel.blend(img.get_pixel(coord.0, coord.1))
                    }

                    //does this work? To make opaque
                    blended_pixel[3] = 255;
                    next_img.put_pixel(x as u32, y as u32, blended_pixel);
                    next_frame[y][x] = CellState::Alive
                }

                _ => {
                    // reduces all remaining pixels' alpha value by decay
                    let pix = next_img.get_pixel_mut(x as u32, y as u32);
                    if let [_r, _g, _b, a] = pix.channels_mut() {
                        if *a >= alpha_decay_per_step {
                            *a -= alpha_decay_per_step;
                        } else {
                            // immediately reduce to invisible if dead
                            *a = 0;
                        }
                    }
                }
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
pub fn neighbors_coords(
    (col, row): (i16, i16),
    cells: &[Vec<CellState>],
    (width, height): (u32, u32),
) -> Vec<(u32, u32)> {
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
    let img_luma = img.to_luma8();
    let cmap = BiLevel;
    let mut palletised = index_colors(&img_luma, &cmap);

    // create cell frame of dead cells
    // create h vectors or w length. index into using vec[y][x]
    let mut cells = vec![vec!(CellState::Dead; w as usize); h as usize];

    match mode {
        Mode::Light => {
            for (x, y, pix) in palletised.enumerate_pixels_mut() {
                let (x, y) = (x as usize, y as usize);
                match pix.0[0] {
                    0 => {}
                    1 => cells[y][x] = CellState::Alive,
                    _ => panic!(),
                }
            }
            cells
        }

        Mode::Dark => {
            for (x, y, pix) in palletised.enumerate_pixels_mut() {
                let (x, y) = (x as usize, y as usize);
                match pix.0[0] {
                    0 => cells[y][x] = CellState::Alive,
                    1 => {}
                    _ => panic!(),
                }
            }
            cells
        }

        _ => {
            panic!()
        }
    }
}
