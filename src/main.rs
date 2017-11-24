
extern crate getopts;

use std::env;
use std::error::Error;
use getopts::Options;

mod fortune;

const FORTUNE_DIR: &'static str = "/usr/share/games/fortunes";

fn main() {

    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optflag("a", "", "choose from all lists of maxims");
    opts.optflag("c", "", "show the cookie file from which the fortune came");
    opts.optflag("e", "", "consider all fortune files to be of equal size");
    opts.optflag("f", "", "print out the list of files to be searched");
    opts.optflag("l", "", "long dictums only");
    opts.optopt("m", "", "print all fortunes matching the regex", "pattern");
    opts.optopt("n", "", "set the longest length considered short", "len");
    opts.optflag("o", "", "choose only from potentially offensive aphorisms");
    opts.optflag("s", "", "short apothegms only");
    opts.optflag("i", "", "ignore case for -m patterns");
    opts.optflag("w", "", "wait  before termination");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    match run(FORTUNE_DIR) {
        Ok(_) => return,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    }
}

fn run(dir: &str) -> Result<(), Box<Error>> {
    let mut fortune = fortune::new();
    try!(fortune.load(dir));

    //fortune.search("success", |x| println!("({})", x), |x| print!("{}", x));
    try!(fortune.get(|x| print!("{}", x)));

    Ok(())
}
