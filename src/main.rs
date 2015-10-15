extern crate fuzzy;
extern crate docopt;
extern crate rustc_serialize;

use fuzzy::initialize;
use docopt::Docopt;

const USAGE: &'static str = "
Fuzzy: the fuzzy file finder.

Usage:
  fuzzy [FLAGS]

Flags:
  -h --help     Show this screen. 
  --auto-copy   Automatically copy result to clipboard.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_auto_copy: bool,
}


fn main() {
    let args: Args = Docopt::new(USAGE)
                             .and_then(|d| d.decode())
                             .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
    initialize();
}
