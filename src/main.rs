// An implementation of fortune(6) in Rust.

extern crate getopts;

use std::cmp::max;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ops::Deref;
use std::thread::sleep;
use std::time::Duration;
use getopts::{Matches, Options};

mod fortune;

const MIN_WAIT     : usize = 6;
const CHARS_PER_SEC: usize = 20;
const FORTUNE_DIR  : &'static str = "/usr/share/games/fortunes";

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
        let brief = format!("Usage: {} [options] [[n%] file/dir]...", args[0]);
        print!("{}", opts.usage(&brief));
        return;
    }

    // Get file list from command line with optional percentages
    let mut filelist = HashMap::new();
    if matches.free.is_empty() {
        filelist.insert(FORTUNE_DIR.to_string(), -1.0);
    } else {
        let mut percentage: f32 = -1.0; 
        for mut m in matches.free.clone() {
            if m.ends_with("%") {
                m.pop();
                percentage = m.parse::<f32>().unwrap(); 
            } else {
                filelist.insert(m, percentage);
                percentage = -1.0;
            }
        }
    }

    match run(filelist, matches) {
        Ok(_) => return,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    }
}

fn run(list: HashMap<String, f32>, matches: Matches) -> Result<(), Box<Error>> {
    let mut fortune = fortune::new();

    // Handle offensive fortune options before loading
    if matches.opt_present("o") {
        fortune = fortune.offensive();
    }

    if matches.opt_present("a") {
        fortune = fortune.all();
    }

    for (key, val) in list {
        try!(fortune.load(&key[..]));
    }

    if matches.opt_present("e") {
        fortune = fortune.equal_size();
    }

    fortune = fortune.normalize_weights();

    if matches.opt_present("f") {
        fortune.print_weights();
        return Ok(());
    }

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

    // Print file from which the fortune came
    if matches.opt_present("c") {
        fortune = fortune.show_file();
    }

    match matches.opt_str("m") {
        Some(pat) => {
            let mut p = pat;
            if matches.opt_present("i") {
                p = format!("(?i:{})", p);
            }
            try!(fortune.search(p.deref()))
        },
        None => {
            let fort_size = try!(fortune.print());
            if matches.opt_present("w") {
                sleep(Duration::from_secs(max(fort_size / CHARS_PER_SEC, MIN_WAIT) as u64));
            }
        },
    }

    Ok(())
}
