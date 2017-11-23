extern crate byteorder;

use self::byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path;

const STRFILE_VERSION      : u32 = 2;
const STRFILE_FLAG_RANDOM  : u32 = 0x1;	// randomized pointers
const STRFILE_FLAG__ORDERED: u32 = 0x2;	// ordered pointers
const STRFILE_FLAG__ROTATED: u32 = 0x4;	// rot-13'd text

// information table

#[derive(Default)]
pub struct Strfile {
    pub version : u32,      // version number
    pub numstr  : u32,      // # of strings in the file
    pub longlen : u32,      // length of longest string
    pub shortlen: u32,      // length of shortest string
    pub flags   : u32,      // bit field for flags
    pub stuff   : [u8; 4],  // long aligned space
    pub seekpts : Vec<u32>  // seek pointers
}

impl Strfile {

    pub fn load(&mut self, path: path::PathBuf) -> Result<(), io::Error> {

        //let file = try!(File::open(filename).map_err(|e| e.to_string()));
        let file = try!(File::open(path));
	let mut f = BufReader::new(&file);

	self.version  = try!(f.read_u32::<BigEndian>());
	self.numstr   = try!(f.read_u32::<BigEndian>());
	self.longlen  = try!(f.read_u32::<BigEndian>());
	self.shortlen = try!(f.read_u32::<BigEndian>());
	self.flags    = try!(f.read_u32::<BigEndian>());
	

        Ok(())
    }
}

