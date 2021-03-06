# Fuzzy

Fuzzy is a simple fuzzy file finder written in Rust.

## Inspiration

I have been playing with Rust for a while now and was looking for something that would benefit from Rust's speed and concurrency capabilities.
I have been using [FZF](https://github.com/junegunn/fzf) and that's where I got the idea from.

## Installation

### Mac OS

Install the latest release for max here: https://github.com/sebglazebrook/fuzzy/releases

Then chuck it in your path and remove the "_osx" suffix on the filename.

### Linux

Install the latest release for max here: https://github.com/sebglazebrook/fuzzy/releases

Then chuck it in your path and remove the "_linux" suffix on the filename.

The binary has been tested on debian jessie.

## Usage

Just type `fuzzy` and press enter.

You'll start to see a list of all files and directories recursivley from your current directory.

Now just start typing to filter the results and find what you need.

When you find what you want press `enter` to exit or `ctrl + y` to copy the result to your clipboard and exit.
