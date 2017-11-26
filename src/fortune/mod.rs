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
    jars      : Vec<strfile::Strfile>, // list of cookie files
}

impl Fortune {

    // Load cookie files metadata
    pub fn load(&mut self, what: &str) -> Result<(), Box<Error>> {

        let mut files: Vec<path::PathBuf> = Vec::new();
        let md = try!(fs::metadata(what));

        if md.is_dir() {
            files = add_fortune_dir(files, what)?;
        } else if md.is_file() {
            files = add_fortune_file(files, what)?;
        }

        if files.len() <= 0 {
            return Err(From::from("No fortunes found".to_string()));
        }

        for f in files {
            let sf = strfile::new();
            self.jars.push(sf.load(&f)?);
        }

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

    // Normalize weights to totalize 100%
    pub fn normalize_weights(mut self) -> Self {
        let mut w: f32 = 0.0;
        for cf in &self.jars {
            w += cf.weight;
        }
        w /= 100.0;
        for cf in &mut self.jars {
            cf.weight /= w;
        }

        self
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

        return range.ind_sample(&mut rng);
    }

    // Get a random string from a random cookie file
    pub fn get<F>(&self, f: F) -> Result<(), Box<Error>> where F: FnOnce(&String) {
        return self.pick_jar().get_one(self.slen, self.long_only, self.short_only, f);
    }

    // Get all strings that match a given regexp pattern
    pub fn search<F1, F2>(&self, pat: &str, fname: F1, fun: F2) -> Result<(), Box<Error>>
        where F1: Fn(&String), F2: Fn(&String) {

        let re = Regex::new(pat).unwrap();

        for cf in &self.jars {
            try!(cf.get_many(&re, self.slen, self.long_only, self.short_only, &fname, &fun));
        }
        Ok(())
    }

    pub fn print_weights(self) {
        for cf in self.jars {
            println!("   {:6.2}% {}", cf.weight, cf.name);
        }
    }
}

pub fn new() -> Fortune {
    return Fortune{
        slen: 160,
        long_only: false,
        short_only: false,
        jars: Vec::new(),
    }
}

fn add_fortune_dir(mut v: Vec<path::PathBuf>, dir: &str) ->
    Result<Vec<path::PathBuf>, io::Error> {

    for entry in fs::read_dir(dir)? {
        let mut path = entry?.path();
        if path.extension().and_then(OsStr::to_str) == Some("dat") {
            // remove file extension
            let p = path.clone();
            let stem = p.file_stem().unwrap();

            path.pop();
            path.push(stem);

            v.push(path)
        }
    }

    Ok(v)
}

fn add_fortune_file(mut v: Vec<path::PathBuf>, name: &str) ->
    Result<Vec<path::PathBuf>, Box<Error>> {

    let datname = String::from(name) + ".dat";
    let md = try!(fs::metadata(&datname));

    if md.is_file() {
        v.push(path::PathBuf::from(datname));
    } else {
        return Err(From::from(format!("{}: missing strfile data file", name)));
    }

    Ok(v)
}

