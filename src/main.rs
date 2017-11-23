
extern crate getopts;

use getopts::Options;
use std::env;
use std::io;
use std::fs;
use std::path;
use std::ffi::OsStr;

mod strfile;

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
        Err(f) => {
            println!("{}", f);
            return;
        }
    };

    let files = get_fortune_files(FORTUNE_DIR);
    let files = files.unwrap();

    println!("{:?}", files);

    let mut path = path::PathBuf::new();
    path.push(FORTUNE_DIR);
    path.push("fortunes.dat");

    let mut dat : strfile::Strfile = Default::default();
    dat.load(path);
    
    println!("{}", dat.version);
}

fn get_fortune_files(dir: &str) -> Result<Vec<path::PathBuf>, io::Error> {

    let mut v: Vec<path::PathBuf> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(OsStr::to_str) == Some("dat") {
            v.push(path)
        }
    }

    Ok(v)
}

