extern crate rand;
extern crate regex;
extern crate rot13;

use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, Seek, SeekFrom};
use std::path;
use self::rand::distributions::{self, Range, IndependentSample};
use self::regex::Regex;

mod strfile;

pub struct Fortune {
    slen: u32,
    long_only: bool,
    short_only: bool,
    jars: Vec<CookieFile>,
}

// Trait to read lines from a file

trait ReadLines {
    fn read_lines(&mut self, s: String, size: usize) -> Result<String, Box<Error>>;
}

impl<R: io::Read> ReadLines for io::BufReader<R> {
    fn read_lines(&mut self, mut s: String, size: usize) -> Result<String, Box<Error>> {
        s.clear();
        while s.len() < size {
            try!(self.read_line(&mut s));
        }
        Ok(s)
    }
}

// Fortune reader

impl Fortune {

    // Load cookie files metadata
    pub fn load(&mut self, dir: &str) -> Result<(), Box<Error>> {
        let files = fortune_files(dir)?;

        if files.len() <= 0 {
            return Err(From::from("No fortunes found".to_string()));
        }

        for f in files {
            self.jars.push(cookie_file(f)?);
        }

        Ok(())
    }

    pub fn long_only(mut self) -> Self {
        self.long_only = true;
        self.short_only = false;
        self
    }

    pub fn short_only(mut self) -> Self{
        self.short_only = true;
        self.long_only = false;
        self
    }

    pub fn short_len(mut self, n: u32) -> Self{
        self.slen = n;
        self
    }

    pub fn equal_size(mut self) -> Self{
        // set all weights to 1 if considering equal size
        for cf in &mut self.jars {
            cf.weight = 1;
        }
        self
    }

    // Choose a random cookie file weighted by its number of strings
    fn pick_jar(&self) -> &CookieFile {

        let mut rng = rand::thread_rng();
        let mut items : Vec<distributions::Weighted<&CookieFile>> = Default::default();

        for cf in &self.jars {
            items.push(distributions::Weighted{
                weight: cf.weight,
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
}

pub fn new() -> Fortune {
    return Fortune{
        slen: 160,
        long_only: false,
        short_only: false,
        jars: Vec::new(),
    }
}

fn fortune_files(dir: &str) -> Result<Vec<path::PathBuf>, io::Error> {

    let mut v: Vec<path::PathBuf> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(OsStr::to_str) == Some("dat") {
            v.push(path)
        }
    }

    Ok(v)
}

// Cookie file

#[derive(Clone)]
struct CookieFile {
    name: OsString,
    path: path::PathBuf,
    weight: u32,
    dat : strfile::Strfile,
}

impl CookieFile {

    fn get_one<F>(&self, slen: u32, long_only: bool, short_only: bool, fun: F) ->
        Result<(), Box<Error>> where F: FnOnce(&String) {

        let range = Range::new(0, self.dat.numstr);
        let mut rng = rand::thread_rng();
        let (mut which, mut start, mut size);
   
        loop {
            which = range.ind_sample(&mut rng) as usize;
            start = self.dat.start_of(which);
            size = self.dat.end_of(which) - start - 2;

            if (!long_only && size <= slen) || (!short_only && size > slen) {
                break;
            }
        }

        let file = try!(File::open(self.path.clone()));
        let mut f = io::BufReader::new(&file);
        try!(f.seek(SeekFrom::Start(start as u64)));

        let mut s = String::with_capacity(size as usize);
        s = try!(f.read_lines(s, size as usize));

        if self.dat.is_rotated() {
            s = rot13::rot13(&s[..]);
        }

        fun(&s);

        Ok(())
    }

    fn get_many<F1, F2>(&self, re: &Regex, slen: u32, long_only: bool, short_only: bool,
        fname: F1, fun: F2) -> Result<(), Box<Error>> where F1: Fn(&String), F2: Fn(&String) {

        use std::ops::Deref;

        let file = try!(File::open(self.path.clone()));
        let mut f = io::BufReader::new(&file);

        let mut s = String::with_capacity(self.dat.longlen as usize);

        fname(&self.name.to_str().unwrap().to_string());

        for n in 0..self.dat.numstr as usize {
            let start = self.dat.start_of(n);
            let size = self.dat.end_of(n) - start - 2;

            s = try!(f.read_lines(s, size as usize));

            if (!long_only && size <= slen) || (!short_only && size > slen) {
                if re.is_match(s.deref()) {
                    fun(&s);
                }
            }
        }

        Ok(())
    }
}

fn cookie_file(mut path: path::PathBuf) -> Result<CookieFile, Box<Error>> {

    let data_path = path.clone();
    let stem = match data_path.file_stem() {
        Some(val) => val,
        None => return Err(From::from("invalid data file".to_string())),
    };

    path.pop();
    path.push(stem);

    let mut cf = CookieFile{
        name: stem.to_os_string(),
        weight: 1,
        path,
        dat : Default::default(),
    };

    try!(cf.dat.load(&data_path));

    cf.weight = cf.dat.numstr;

    Ok(cf)
}

