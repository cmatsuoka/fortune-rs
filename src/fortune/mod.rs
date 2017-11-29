extern crate rand;
extern crate regex;

use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path;
use self::rand::distributions::{self, IndependentSample};
use self::regex::Regex;

mod strfile;

// Fortune reader

pub struct Fortune {
    slen      : u32,                   // short fortune length
    long_only : bool,                  // display only long fortunes
    short_only: bool,                  // display only short fortunes
    show_file : bool,                  // display the cookie file name
    all_forts : bool,                  // allow offensive fortunes
    offend    : bool,                  // choose only from offensive fortunes
    jars      : Vec<strfile::Strfile>, // list of cookie files
}

type Pathspec = (path::PathBuf, f32);

impl Fortune {

    // Load cookie files metadata
    pub fn load(&mut self, what: &str, val: f32) -> Result<(), Box<Error>> {
        let mut files: Vec<Pathspec> = Vec::new();
        let md = try!(fs::metadata(what));

        if md.is_dir() {
            files = add_fortune_dir(files, what, val, self.all_forts, self.offend)?;
        } else if md.is_file() {
            files = add_fortune_file(files, what, val)?;
        }

        if files.len() <= 0 {
            return Err(From::from("No fortunes found".to_string()));
        }

        for f in files {
            let sf = strfile::new();
            self.jars.push(sf.load(&f.0, f.1)?);
        }

        self.jars.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(())
    }

    // Select only long messages
    pub fn long_only(mut self) -> Self {
        self.long_only = true;
        self.short_only = false;
        self
    }

    // Select only short messages
    pub fn short_only(mut self) -> Self {
        self.short_only = true;
        self.long_only = false;
        self
    }

    // Set the short message threshold
    pub fn short_len(mut self, n: u32) -> Self {
        self.slen = n;
        self
    }

    // Set all weights to 1 if considering equal size
    pub fn equal_size(mut self) -> Self {
        for cf in &mut self.jars {
            cf.weight = 1.0;
        }
        self
    }

    // Show the file where the fortune came from
    pub fn show_file(mut self) -> Self {
        self.show_file = true;
        self
    }

    // Allow both offensive and not offensive fortunes
    pub fn all(mut self) -> Self {
        self.all_forts = true;
        self.offend = false;
        self
    }

    // Select only from offensive fortunes
    pub fn offensive(mut self) -> Self {
        self.all_forts = false;
        self.offend = true;
        self
    }

    // Normalize weights to totalize 100%
    pub fn normalize_weights(mut self) -> Result<Self, Box<Error>> {
        let mut total: f32 = 100.0;

        for cf in &self.jars {
            if cf.weight > 0.0 {
                total -= cf.weight;
            }
        }

        if total < 0.0 {
            return Err(From::from("percentages must be <= 100".to_string()));
        }

        let mut w: f32 = 0.0;
        for cf in &self.jars {
            if cf.weight < 0.0 {
                w += cf.num_str() as f32;
            }
        }
        for cf in &mut self.jars {
            if cf.weight < 0.0 {
                cf.weight = total * cf.num_str() as f32 / w;
            }
        }

        Ok(self)
    }

    // Choose a random cookie file weighted by its number of strings
    fn pick_jar(&self) -> &strfile::Strfile {

        let mut rng = rand::thread_rng();
        let mut items : Vec<distributions::Weighted<&strfile::Strfile>> = Default::default();

        for cf in &self.jars {
            items.push(distributions::Weighted{
                weight: (cf.weight * 100.0) as u32,
                item  : cf,
            });
        }

        let range = distributions::WeightedChoice::new(&mut items);

        range.ind_sample(&mut rng)
    }

    // Get a random string from a random cookie file
    pub fn print(&self) -> Result<usize, Box<Error>> {
        loop {
            match self.pick_jar().print_one(self.slen, self.long_only, self.short_only, self.show_file) {
                Ok(val) => {
                    if val > 0 {
                        return Ok(val);
                    }
                },
                Err(e) => return Err(e),
            }
        }
    }

    // Get all strings that match a given regexp pattern
    pub fn search(&self, pat: &str) -> Result<(), Box<Error>> {
        let re = Regex::new(pat).unwrap();

        for cf in &self.jars {
            try!(cf.print_matches(&re, self.slen, self.long_only, self.short_only));
        }
        Ok(())
    }

    pub fn print_weights(self) {
        for cf in self.jars {
            let info = cf.info();
            println!(" {:6.2}% {:5} {:5} {:5} {}", cf.weight, cf.num_str(), info.1, info.2, cf.filepath());
        }
    }
}

pub fn new() -> Fortune {
    return Fortune{
        slen      : 160,
        long_only : false,
        short_only: false,
        show_file : false,
        all_forts : false,
        offend    : false,
        jars      : Vec::new(),
    }
}

fn add_fortune_dir(mut v: Vec<Pathspec>, dir: &str, val: f32, all_forts: bool,
    offend: bool) -> Result<Vec<Pathspec>, io::Error> {

    let mut list: Vec<path::PathBuf> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let mut path = entry?.path();
        if path.extension().and_then(OsStr::to_str) == Some("dat") {
            // remove file extension
            let p = path.clone();
            let stem = p.file_stem().unwrap();
            let name = stem.to_str().unwrap().to_string();

            if all_forts || !(offend ^ name.ends_with("-o")) {
                path.pop();
                path.push(stem);
                list.push(path)
            }
        }
    }

    let num = list.len() as f32;
    for l in list.clone() {
        v.push((l, val / num));
    }

    Ok(v)
}

fn add_fortune_file(mut v: Vec<Pathspec>, name: &str, val: f32) ->
    Result<Vec<Pathspec>, Box<Error>> {

    let datname = String::from(name) + ".dat";
    let md = try!(fs::metadata(&datname));

    if md.is_file() {
        v.push((path::PathBuf::from(name), val));
    } else {
        return Err(From::from(format!("{}: missing strfile data file", name)));
    }

    Ok(v)
}

