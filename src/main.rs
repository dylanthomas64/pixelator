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
    /// output directory. CWD if none given.
    output_directory: Option<String>,

    #[arg(short, long, default_value_t = 100)]
    ///number of steps in the game of life
    generations: u64,

    /// 0-255, how much the colours decay each step. Deducts from alpha values each step.
    #[arg(short, long, default_value_t = 32)]
    decay: u8,

    #[arg(short, long)]
    /// select "dark" or "light" mode for colour mapping options. Default = "random"
    mode: Option<String>,

    /// set true for faster lossy gif rendering
    #[arg(short, long, default_value_t = false)]
    speed: bool,

    /// output pixel width dimension using pixelation effect. Default is original image size.
    /// smaller picture sizes result in exponentially faster renders.
    #[arg(short, long)]
    width: Option<u32>,
}

fn main() {
    let args = Args::parse();
    println!("path : {}", args.path);

    let image_path = args.path;
    let generations = args.generations;
    let decay = args.decay;

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

    println!("opening image...");
    let mut img = image::open(image_path).expect("failed to load image");

    if let Some(width) = args.width {
        img = pixelate(img, width);
    }

    let slides = begin_life(img, generations, decay, &mode);
    let blended = background_for_slides(slides, BackgroundColour::default());

    make_gif(blended, speed, &output_path);
}

fn create_directory(path: &str) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}
