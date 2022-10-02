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


./pixelator -p "./images/jupiter.png"

./pixelator -p "./images/jupiter.png" -o "./test" -w 120 -g 500 -b "w"

./pixelator -p "./images/jupiter.png" -o "./test" -w 480 -g 20 -b "#1BEC84" -m "light"

./pixelator -p "./images/beetroot.jpg" -w 560 -g 200 -b "black"

./target/release/pixelator  /home/Dylan/Pictures/shyguy.png -o /home/dylan/Pictures/cgol



