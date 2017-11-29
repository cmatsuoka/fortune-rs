extern crate byteorder;
extern crate rand;
extern crate regex;
extern crate rot13;

use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Seek, SeekFrom};
use std::path;
use std::str;
use self::byteorder::{BigEndian, ReadBytesExt};
use self::rand::distributions::{self, IndependentSample};
use self::regex::Regex;


const STRFILE_VERSION     : u32 = 2;
const STRFILE_FLAG_RANDOM : u32 = 0x1;	// randomized pointers
const STRFILE_FLAG_ORDERED: u32 = 0x2;	// ordered pointers
const STRFILE_FLAG_ROTATED: u32 = 0x4;	// rot-13'd text

// String file

#[derive(Clone, Default)]
pub struct Strfile {
    pub name  : String,        // cookie file name
    pub weight: f32,           // weight of this file for random pick
    path      : path::PathBuf, // path to strfile metadata file
    dat       : Datfile,       // strfile metadata
}

impl Strfile {

    pub fn load(mut self, path: &path::PathBuf, weight: f32) -> Result<Strfile, Box<Error>> {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();

        // check if file exists
        let md = try!(fs::metadata(path));
        if !md.is_file() {
            return Err(From::from(format!("{}: invalid file type", name)));
        }
 
        let mut dat_path = path.clone();
        dat_path.set_extension("dat");

        try!(self.dat.load(&dat_path));
    
        self.name = name;
        self.path = path.clone();
        self.weight = weight; //self.dat.numstr as f32;
    
        Ok(self)
    }

    pub fn num_str(&self) -> usize {
        self.dat.numstr as usize
    }

    pub fn info(&self) -> (usize, usize, usize, u32) {
        let d = &self.dat;
        (d.version as usize, d.longlen as usize, d.shortlen as usize, d.flags)
    }

    pub fn filepath(&self) -> &str {
        self.path.to_str().unwrap()
    }

    pub fn print_one(&self, slen: u32, long_only: bool, short_only: bool, show_file: bool) ->
        Result<usize, Box<Error>> {

        let range = distributions::Range::new(0, self.dat.numstr);
        let mut rng = rand::thread_rng();
   
        let which = range.ind_sample(&mut rng) as usize;
        let start = self.dat.start_of(which);
        let size = self.dat.end_of(which) - start - 2;

        if (long_only || size > slen) && (short_only || size <= slen) {
            return Ok(0);
        }

        let file = try!(File::open(self.path.clone()));
        let mut f = io::BufReader::new(&file);
        try!(f.seek(SeekFrom::Start(start as u64)));

        let mut s = String::with_capacity(size as usize);
        s = try!(f.read_lines_until(s, &self.separator().to_string()));

        if self.dat.is_rotated() {
            s = rot13::rot13(&s[..]);
        }

        if show_file {
            println!("({})\n{}", self.name, self.separator());
        }

        print!("{}", s);

        Ok(s.len())
    }

    pub fn print_matches(&self, re: &Regex, slen: u32, long_only: bool, short_only: bool) ->
        Result<(), Box<Error>> {

        let file = try!(File::open(self.path.clone()));
        let mut f = io::BufReader::new(&file);
        let mut v: Vec<u8> = Vec::with_capacity(self.dat.longlen as usize);

        eprintln!("({})\n{}", self.name, self.separator());

        for n in 0..self.dat.numstr as usize {
            let start = self.dat.start_of(n);
            let size = self.dat.end_of(n) - start;

            unsafe { v.set_len(size as usize); }
            try!(f.read_exact(&mut v[..]));

            if (!long_only && size <= slen) || (!short_only && size > slen) {
                let s = String::from_utf8_lossy(&v);
                if re.is_match(&s) {
                    print!("{}", s);
                }
            }
        }

        Ok(())
    }

    fn separator(&self) -> char {
        self.dat.stuff[0] as char
    }
}

pub fn new() -> Strfile {
    return Default::default();
}


// Dat file

#[derive(Default, Clone)]
struct Datfile {
    version : u32,      // version number
    numstr  : u32,      // # of strings in the file
    longlen : u32,      // length of longest string
    shortlen: u32,      // length of shortest string
    flags   : u32,      // bit field for flags
    stuff   : [u8; 4],  // long aligned space
    seekpts : Vec<u32>  // seek pointers
}

impl Datfile {

    pub fn load(&mut self, path: &path::PathBuf) -> Result<(), Box<Error>> {

        let file = try!(File::open(path));
        let mut f = io::BufReader::new(&file);

        self.version  = try!(f.read_u32::<BigEndian>());

        if self.version > STRFILE_VERSION {
            return Err(From::from("invalid data file version".to_string()));
        }

        self.numstr   = try!(f.read_u32::<BigEndian>());
        self.longlen  = try!(f.read_u32::<BigEndian>());
        self.shortlen = try!(f.read_u32::<BigEndian>());
        self.flags    = try!(f.read_u32::<BigEndian>());
        try!(f.read_exact(&mut self.stuff));

        for _ in 0..(self.numstr + 1) {
            self.seekpts.push(try!(f.read_u32::<BigEndian>()));
        }

        // if sorted or random, sort seek pointers
        if self.flags & (STRFILE_FLAG_RANDOM | STRFILE_FLAG_ORDERED) != 0 {
            self.seekpts.sort();
        }

        Ok(())
    }

    #[inline]
    pub fn start_of(&self, which: usize) -> u32 {
        self.seekpts[which]
    }

    #[inline]
    pub fn end_of(&self, which: usize) -> u32 {
        self.seekpts[which + 1]
    }

    #[inline]
    pub fn is_rotated(&self) -> bool {
        self.flags & STRFILE_FLAG_ROTATED != 0
    }
}

// Trait to read lines from a file

trait ReadLines {
    fn read_lines_until(&mut self, s: String, sep: &str) -> Result<String, Box<Error>>;
}

impl<R: io::Read> ReadLines for io::BufReader<R> {
    fn read_lines_until(&mut self, mut s: String, sep: &str) -> Result<String, Box<Error>> {
        let mut buf = String::new();
        s.clear();
        loop {
            buf.clear();
            let n = try!(self.read_line(&mut buf));
            if n == 0 || &*buf.trim() == sep {
                break;
            }
            s.push_str(&buf);
        }
        Ok(s)
    }
}
