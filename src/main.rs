
extern crate getopts;

use std::env;
use std::error::Error;
use std::ops::Deref;
use getopts::{Matches, Options};

mod fortune;

const FORTUNE_DIR: &'static str = "/usr/share/games/fortune";

fn main() {

    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optflag("a", "", "choose from all lists of maxims");
    opts.optflag("c", "", "show the cookie file from which the fortune came");
    opts.optflag("e", "", "consider all fortune files to be of equal size");
    opts.optflag("f", "", "print out the list of files to be searched");
    opts.optflag("h", "help", "display usage information and exit");
    opts.optflag("l", "", "long dictums only");
    opts.optopt("m", "", "print all fortunes matching the regex", "pattern");
    opts.optopt("n", "", "set the longest length considered short", "len");
    opts.optflag("o", "", "choose only from potentially offensive aphorisms");
    opts.optflag("s", "", "short apothegms only");
    opts.optflag("i", "", "ignore case for -m patterns");
    opts.optflag("w", "", "wait before termination");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options] [[n%] file/dir ...]", args[0]);
        print!("{}", opts.usage(&brief));
        return;
    }

    match run(FORTUNE_DIR, matches) {
        Ok(_) => return,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    }
}

fn run(dir: &str, matches: Matches) -> Result<(), Box<Error>> {
    let mut fortune = fortune::new();

    try!(fortune.load(dir));

    // Handle option to set the short fortune threshold
    match matches.opt_str("n") {
        Some(val) => {
            fortune = fortune.short_len(val.parse()?);
        },
        None => (),
    }

    // Handle long- and short-only switch
    if matches.opt_present("l") {
        fortune = fortune.long_only();
    }

    if matches.opt_present("s") {
        fortune = fortune.short_only();
    }

    if matches.opt_present("e") {
        fortune = fortune.equal_size();
    }

    match matches.opt_str("m") {
        Some(pat) => {
            let mut p = pat;
            if matches.opt_present("i") {
                p = format!("(?i:{})", p);
            }
            let p = p.deref();
            try!(fortune.search(p, |x| println!("({})", x), |x| print!("{}", x)))
        },
        None => {
            try!(fortune.get(|x| print!("{}", x)))
        },
    }

    Ok(())
}
