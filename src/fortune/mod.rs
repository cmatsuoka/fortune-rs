extern crate regex;

use std::error::Error;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path;
use self::regex::Regex;

mod strfile;

pub struct Fortune {
    jars: Vec<CookieJar>,
}

impl Fortune {

    pub fn load(&mut self, dir: &str) -> Result<(), Box<Error>> {
        for f in fortune_files(dir)? {
            self.jars.push(cookie_jar(f)?);
        }

        Ok(())
    }

    pub fn get<F>(&self, f: F) -> Result<(), Box<Error>>
        where F: FnOnce(String) {
        try!(self.jars[0].get_one(0, f));
        Ok(())
    }
}

pub fn new() -> Fortune {
    return Fortune{
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


struct CookieJar {
    path: path::PathBuf,
    dat : strfile::Strfile,
}

impl CookieJar {

    fn get_one<F>(&self, which: usize, f: F) -> Result<(), Box<Error>>
        where F: FnOnce(String) {

        let start = self.dat.seekpts[which] as u64;
        let end = self.dat.seekpts[which + 1] as u64;
        let size = end - start - 2;

        let mut file = try!(File::open(self.path.clone()));
        try!(file.seek(SeekFrom::Start(start)));

        let mut buf = vec![0_u8; size as usize];
        try!(file.read_exact(buf.as_mut_slice()));

        f(String::from_utf8(buf).unwrap());

        Ok(())
    }

    fn get_many<F>(&self, pat: &str) -> Result<(), Box<Error>>
        where F: FnOnce(String) {

        let re = Regex::new(pat).unwrap();
        let mut file = try!(File::open(self.path.clone()));

        for n in 0..self.dat.numstr {
            let start = self.dat.seekpts[n as usize] as u64;
            let end = self.dat.seekpts[n as usize + 1] as u64;
            let size = end - start - 2;
        }

        Ok(())
    }
}

fn cookie_jar(mut path: path::PathBuf) -> Result<CookieJar, Box<Error>> {

    let data_path = path.clone();
    let stem = match data_path.file_stem() {
        Some(val) => val,
        None => return Err(From::from("invalid data file".to_string())),
    };

    path.pop();
    path.push(stem);

    let mut jar = CookieJar{
        path,
        dat : Default::default(),
    };

    try!(jar.dat.load(&data_path));

    Ok(jar)
}

