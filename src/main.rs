use std::{fs, str::FromStr};

use pixelator::{background_for_slides, make_gif, pixelate, BackgroundColour, Mode};

mod conway;
use conway::begin_life;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about= "does stuff", long_about = None)]
struct Args {
    #[arg(short, long)]
    /// path to image
    path: String,

    #[arg(short, long)]
    /// Output directory. ./output/ if none given.
    output_directory: Option<String>,

    #[arg(short, long, default_value_t = 100)]
    /// Number of steps in the game of life
    generations: u64,

    #[arg(short, long, default_value_t = 32)]
    /// How much alpha values reduce each step.
    decay: u8,

    #[arg(short, long, default_value = "b")]
    /// "white"/"w", "black"/"b", "#RRGGBB"
    background: String,

    #[arg(short, long)]
    /// Select "dark" or "light" mode for colour mapping options. Default = "random"
    mode: Option<String>,

    /// Set true for faster lossy gif rendering
    #[arg(short, long, default_value_t = false)]
    speed: bool,

    /// Output pixel width dimension using pixelation effect. Default is original image size.
    /// Smaller picture sizes result in exponentially faster renders.
    #[arg(short, long)]
    width: Option<u32>,
}

fn main() {
    let args = Args::parse();

    let image_path = args.path;
    let generations = args.generations;
    let decay = args.decay;

    let background: BackgroundColour = args.background.parse().unwrap();

    // parse mode input string
    let mode = match args.mode {
        Some(mode_string) => Mode::from_str(&mode_string).expect("not a valid mode"),
        // default = random
        None => Mode::Random,
    };

    let speed = args.speed;
    // strip filename from path
    let s = image_path.split(['/', '\\', '.']).collect::<Vec<&str>>();
    // take second to last as last element is format
    let file_name = s[s.len() - 2];

    //parse output width as string
    let output_width = match args.width {
        Some(width) => width.to_string(),
        None => "OG".to_string(),
    };

    // create new filename
    let new_file_name = format!(
        "/{}x{}_{}_{}_{}",
        file_name,
        output_width,
        mode.to_string(),
        generations,
        decay
    );

    // create output path if it doesnt already exist
    let output_path = if let Some(path) = args.output_directory {
        create_directory(&path).expect("couldn't create path");
        path + &new_file_name
    } else {
        create_directory("output").expect("couldn't create path");
        "output".to_owned() + &new_file_name
    };

    println!("loading image...");
    let mut img = image::open(image_path).expect("failed to load image");

    if let Some(width) = args.width {
        img = pixelate(img, width);
    }

    let slides = begin_life(img, generations, decay, &mode);
    let blended = background_for_slides(slides, background);

    make_gif(blended, speed, &output_path);
}

fn create_directory(path: &str) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}
