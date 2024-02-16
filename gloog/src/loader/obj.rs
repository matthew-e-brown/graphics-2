use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use thiserror::Error;

use crate::math::{ParseVecError, Vec2, Vec3, Vec4};


// cspell:words curv interp stech ctech usemtl mtllib

// https://paulbourke.net/dataformats/obj/
// https://paulbourke.net/dataformats/mtl/


#[derive(Error, Debug)]
pub enum ObjError {
    #[error("failed to read from file:\n{0:?}")]
    IOError(#[from] io::Error),

    #[error("invalid {0} directive on line {1}")]
    InvalidDirective(&'static str, usize),

    #[error("unsupported directive: {0}")]
    UnsupportedDirective(String),
}


pub struct Model {}

#[derive(Debug, Clone, Copy)]
enum Vertex {
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}


pub fn load<P: AsRef<Path>>(path: P) -> Result<Model, ObjError> {
    let obj_file = BufReader::new(File::open(path)?);

    let mut v = Vec::new();
    let mut vp = Vec::new();
    let mut vn = Vec::new();
    let mut vt = Vec::new();

    let mut lines = obj_file.lines().enumerate();
    while let Some((l, line)) = lines.next() {
        let line = line?;
        let line = trim_line(&line); // also handles comments

        if line.len() == 0 {
            continue;
        }

        // Can unwrap because we just checked the line was non-zero length *after* trimming
        let directive = line.split_whitespace().nth(0).unwrap();

        match directive {
            // vertex data
            "v" => v.push(parse_vertex(&line[1..]).map_err(|_| ObjError::InvalidDirective("v", l))?),
            "vp" => vp.push(parse_vertex(&line[2..]).map_err(|_| ObjError::InvalidDirective("vp", l))?),
            "vn" => vn.push(parse_vertex(&line[2..]).map_err(|_| ObjError::InvalidDirective("vn", l))?),
            "vt" => vt.push(parse_vertex(&line[2..]).map_err(|_| ObjError::InvalidDirective("vt", l))?),
            // elements
            "p" => {},
            "l" => {},
            "f" => {},
            "curv" => {},
            "curv2" => {},
            "surf" => {},
            // grouping
            "g" => {},
            "s" => {},
            "mg" => {},
            "o" => {},
            // display/render attributes
            "bevel" => {},
            "c_interp" => {},
            "d_interp" => {},
            "lod" => {},
            "usemtl" => {},
            "mtllib" => {},
            "shadow_obj" => {},
            "trace_obj" => {},
            "ctech" => {},
            "stech" => {},
            //
            other => return Err(ObjError::UnsupportedDirective(other.to_owned())),
        }
    }

    todo!()
}


fn trim_line(mut s: &str) -> &str {
    s = s.trim_start();
    s = &s[s.find('#').map(|i| 0..i).unwrap_or(0..s.len())]; // find first `#` and only include up to it
    s.trim_end()
}


fn parse_vertex(s: &str) -> Result<Vertex, ParseVecError> {
    match s.trim().split_whitespace().count() {
        n @ (0 | 1) => Err(ParseVecError::TooFewComponents(n as _, 2)),
        2 => Ok(Vertex::Vec2(s.parse()?)),
        3 => Ok(Vertex::Vec3(s.parse()?)),
        4 => Ok(Vertex::Vec4(s.parse()?)),
        n => Err(ParseVecError::TooManyComponents(n as _)),
    }
}
