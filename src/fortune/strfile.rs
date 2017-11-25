extern crate byteorder;

use self::byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path;

const STRFILE_VERSION     : u32 = 2;
const STRFILE_FLAG_RANDOM : u32 = 0x1;	// randomized pointers
const STRFILE_FLAG_ORDERED: u32 = 0x2;	// ordered pointers
const STRFILE_FLAG_ROTATED: u32 = 0x4;	// rot-13'd text

// information table

#[derive(Default, Clone)]
pub struct Strfile {
    pub version : u32,      // version number
    pub numstr  : u32,      // # of strings in the file
    pub longlen : u32,      // length of longest string
    pub shortlen: u32,      // length of shortest string
    flags       : u32,      // bit field for flags
    stuff       : [u8; 4],  // long aligned space
    seekpts     : Vec<u32>  // seek pointers
}

impl Strfile {

    pub fn load(&mut self, path: &path::PathBuf) -> Result<(), io::Error> {

        let file = try!(File::open(path));
        let mut f = BufReader::new(&file);

        self.version  = try!(f.read_u32::<BigEndian>());
        self.numstr   = try!(f.read_u32::<BigEndian>());
        self.longlen  = try!(f.read_u32::<BigEndian>());
        self.shortlen = try!(f.read_u32::<BigEndian>());
        self.flags    = try!(f.read_u32::<BigEndian>());
        try!(f.read_exact(&mut self.stuff));

        for _ in 0..(self.numstr + 1) {
            self.seekpts.push(try!(f.read_u32::<BigEndian>()));
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
}

