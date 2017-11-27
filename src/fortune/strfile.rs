extern crate byteorder;
extern crate rand;
extern crate regex;
extern crate rot13;

use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Seek, SeekFrom};
use std::path;
use self::byteorder::{BigEndian, ReadBytesExt};
use self::rand::distributions::{self, IndependentSample};
use self::regex::Regex;


const STRFILE_VERSION     : u32 = 2;
const STRFILE_FLAG_RANDOM : u32 = 0x1;	// randomized pointers
const STRFILE_FLAG_ORDERED: u32 = 0x2;	// ordered pointers
const STRFILE_FLAG_ROTATED: u32 = 0x4;	// rot-13'd text

// String file

#[derive(Clone,Default)]
pub struct Strfile {
    pub name  : String,        // cookie file name
    pub weight: f32,           // weight of this file for random pick
    path      : path::PathBuf, // path to strfile metadata file
    dat       : Datfile,       // strfile metadata
}

impl Strfile {

    pub fn load(mut self, path: &path::PathBuf) -> Result<Strfile, Box<Error>> {
    
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
        self.weight = self.dat.numstr as f32;
    
        Ok(self)
    }

    pub fn get_one<F>(&self, slen: u32, long_only: bool, short_only: bool, show_file: bool, fun: F) ->
        Result<(), Box<Error>> where F: FnOnce(&String) {

        let range = distributions::Range::new(0, self.dat.numstr);
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

        if show_file {
            println!("({})\n{}", self.name, self.separator());
        }

        fun(&s);

        Ok(())
    }

    pub fn get_many<F1, F2>(&self, re: &Regex, slen: u32, long_only: bool, short_only: bool,
        fname: F1, fun: F2) -> Result<(), Box<Error>> where F1: Fn(&String), F2: Fn(&String) {

        use std::ops::Deref;

        let file = try!(File::open(self.path.clone()));
        let mut f = io::BufReader::new(&file);

        let mut s = String::with_capacity(self.dat.longlen as usize);

        fname(&self.name);

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

