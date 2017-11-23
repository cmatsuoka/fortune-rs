use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path;

mod strfile;

pub struct Fortune {
    jars: Vec<CookieJar>,
}

impl Fortune {

    pub fn load(&mut self, dir: &str) -> Result<(), io::Error> {
        for f in fortune_files(dir)? {
            self.jars.push(cookie_jar(f)?);
        }

        Ok(())
    }

    pub fn get(&self) -> Result<String, io::Error> {
        return self.jars[0].get(0);
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

    fn get(&self, which: usize) -> Result<String, io::Error> {

        let start = self.dat.seekpts[which] as u64;
        let end = self.dat.seekpts[which + 1] as u64;
        let size = end - start - 2;

        let mut file = try!(File::open(self.path.clone()));
        try!(file.seek(SeekFrom::Start(start)));

        let mut buf = vec![0_u8; size as usize];
        try!(file.read_exact(buf.as_mut_slice()));

        let fort = String::from_utf8(buf).unwrap();

        Ok(fort)
    }
}

fn cookie_jar(mut path: path::PathBuf) -> Result<CookieJar, io::Error> {

    let path_clone = path.clone();
    let stem = path_clone.file_stem().unwrap();

    path.pop();
    path.push(stem);

    let mut jar = CookieJar{
        path: path,
        dat : Default::default(),
    };

    let jar_path = path_clone.clone();
    try!(jar.dat.load(jar_path));

    Ok(jar)
}

