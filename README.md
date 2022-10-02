# pixelator
turns an image into a gif of conway's game of life

## Build
$ cargo build --release

## Usage
$ ./target/release/pixelator --help

Usage: pixelator.exe [OPTIONS] --path <PATH>

Options:

  -p, --path <PATH>
          path to image
          
  -o, --output-directory <OUTPUT_DIRECTORY>
          Output directory. ./output/ if none given
          
  -g, --generations <GENERATIONS>
          Number of steps in the game of life [default: 100]
          
  -d, --decay <DECAY>
          How much alpha values reduce each step [default: 32]
          
  -b, --background <BACKGROUND>
          "white"/"w", "black"/"b", "#RRGGBB" [default: b]
          
  -m, --mode <MODE>
          Select "dark" or "light" mode for colour mapping options. Default = "random"
          
  -s, --speed
          Set true for faster lossy gif rendering
          
  -w, --width <WIDTH>
          Output pixel width dimension using pixelation effect. Default is original image size. Smaller picture sizes result in exponentially faster renders
          
  -h, --help
          Print help information
          
  -V, --version
          Print version information
          
          
## Examples


./target/release/pixelator -p "./images/jupiter.png" -o "./example_gifs/" -w 100

https://github.com/dylanthomas64/pixelator/example_gifs/jupiterx100_random_100_32.gif


./target/release/pixelator -p "./images/shyguy.png" -o "./example_gifs/"
  
https://github.com/dylanthomas64/pixelator/blob/6ba3b5edb9a824b05ca28d77ff4e7b253a5a282a/example_gifs/shyguyxOG_random_100_32.gif



