extern crate rand;
extern crate regex;

use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Seek, SeekFrom};
use std::path;
use self::rand::distributions::{self, Range, IndependentSample};
use self::regex::Regex;

mod strfile;

pub struct Fortune {
    pub slen: u32,
    pub long_only: bool,
    pub short_only: bool,
    jars: Vec<CookieFile>,
}

impl Fortune {

    // Load cookie files metadata
    pub fn load(&mut self, dir: &str) -> Result<(), Box<Error>> {
        let files = fortune_files(dir)?;

        if files.len() <= 0 {
            return Err(From::from("no cookie files found".to_string()));
        }

        for f in files {
            self.jars.push(cookie_file(f)?);
        }

        Ok(())
    }

    // Choose a random cookie file weighted by its number of strings
    fn pick_jar(&self) -> &CookieFile {
        let mut items : Vec<distributions::Weighted<&CookieFile>> = Default::default();

        for cf in &self.jars {
            let item = distributions::Weighted{
                weight: cf.dat.numstr,
                item  : cf,
            };

            items.push(item);
        }

        let wc = distributions::WeightedChoice::new(&mut items);
        let mut rng = rand::thread_rng();

        return wc.ind_sample(&mut rng);
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
            try!(cf.get_many(&re, &fname, &fun));
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


#[derive(Clone)]
struct CookieFile {
    name: OsString,
    path: path::PathBuf,
    dat : strfile::Strfile,
}

impl CookieFile {

    fn get_one<F>(&self, slen: u32, long_only: bool, short_only: bool, fun: F) ->
        Result<(), Box<Error>> where F: FnOnce(&String) {

        let range = Range::new(0, self.dat.numstr);
        let mut rng = rand::thread_rng();
        let mut which = 0 as usize;
        let (mut start, mut end, mut size) = (0_u32, 0_u32, 0_u32);
   
        loop {
            which = range.ind_sample(&mut rng) as usize;
            start = self.dat.seekpts[which];
            end = self.dat.seekpts[which + 1];
            size = end - start - 2;

            if (!long_only && size <= slen) || (!short_only && size > slen) {
                break;
            }
        }

        let mut file = try!(File::open(self.path.clone()));
        try!(file.seek(SeekFrom::Start(start as u64)));

        let mut buf = vec![0_u8; size as usize];
        try!(file.read_exact(buf.as_mut_slice()));

        let s = String::from_utf8(buf).unwrap();
        fun(&s);

        Ok(())
    }

    fn get_many<F1, F2>(&self, re: &Regex, fname: F1, fun: F2) -> Result<(), Box<Error>>
        where F1: Fn(&String), F2: Fn(&String) {

        use std::ops::Deref;

        let mut file = try!(File::open(self.path.clone()));
        let mut f = io::BufReader::new(&file);

        let mut s = String::with_capacity(self.dat.longlen as usize);

        fname(&self.name.to_str().unwrap().to_string());

        for n in 0..self.dat.numstr {
            let start = self.dat.seekpts[n as usize] as u64;
            let end = self.dat.seekpts[n as usize + 1] as u64;
            let size = end - start - 2;

            s.truncate(0);

            while s.len() < size as usize {
                f.read_line(&mut s);
            }

            if re.is_match(s.deref()) {
                fun(&s);
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
        path,
        dat : Default::default(),
    };

    try!(cf.dat.load(&data_path));

    Ok(cf)
}

