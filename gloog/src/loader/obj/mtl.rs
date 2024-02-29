#![allow(unused)] // just till we get texture caches figured out

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use gloog_math::Vec3;
use image::ImageResult;
use log::{debug, info, warn};

use super::error::{MtlLoadError, MtlResult};
use super::{read_ws_verts, trim_comment, CachedImage, ObjMaterial};
use crate::loader::{lines_escaped, LineRange};

// cspell:words newmtl usemtl

pub fn parse_mtl_file(
    path: impl AsRef<Path>,
    parsed_materials: &mut Vec<ObjMaterial>,
    material_indices: &mut HashMap<Box<str>, usize>,
    texture_cache: &mut HashMap<Box<str>, CachedImage>,
) -> MtlResult<()> {
    let file = File::open(path).map_err(|err| MtlLoadError::IOOpenError(err))?;
    let mut lines = lines_escaped(BufReader::new(file));

    let mut cur_name = None; // Name of the material we're currently parsing lines for
    let mut skip_mode = false; // Whether or not we're currently ignoring non-`newmtl` lines

    // Loop once for each material
    'materials: loop {
        // New empty material for the next entry; we loop through lines until we find a `newmtl` that sets this name.
        let mut material = ObjMaterial::default();

        /// Sets a field in `material`, logging a warning/info message if it already has a value.
        macro_rules! set_single {
            ($prop:literal, $mtl:ident . $field:ident = $value:expr) => {{
                if material.$field.is_some() {
                    debug!("multiple values given for {p} in material {m}; skipping.", p = $prop, m = $mtl);
                }

                let _ = material.$field.insert($value);
            }};
        }

        while let Some(line_result) = lines.next() {
            let (line_nums, line) = match line_result {
                Ok(line) => line,
                Err(err) => return Err(MtlLoadError::IOReadError(err)),
            };

            let line = trim_comment(&line);
            if line.len() == 0 {
                continue;
            }

            let directive = line.split_whitespace().nth(0).unwrap();
            let line = line[directive.len()..].trim_start(); // rest of line

            // When we find the start of a new material:
            // - If we have a current material, it is finished and can be inserted.
            // - We are about to start the next one, so we need to check if it's already in the map.
            if directive == "newmtl" {
                // Found a new material, can stop skipping
                skip_mode = false;

                if let Some(name) = cur_name {
                    info!("parsed material {name}: {material:?}");
                    parsed_materials.push(material);
                    material_indices.insert(name, parsed_materials.len() - 1);
                }

                // As with `usemtl`, we can just take the whole rest of the line.
                let new_name: Box<str> = line.into();

                if material_indices.contains_key(&new_name) {
                    skip_mode = true; // We can skip the next material
                    cur_name = None; // Don't need a name
                } else {
                    // Set next name
                    cur_name = Some(new_name);
                }

                continue 'materials;
            } else if skip_mode {
                // Do nothing, skip this field till we find another `new_mtl`
                continue;
            } else if let Some(mtl) = cur_name.as_deref() {
                match directive {
                    "Kd" => set_single!("Kd", mtl.diffuse = parse_color(line, &line_nums)?),
                    "Ka" => set_single!("Ka", mtl.ambient = parse_color(line, &line_nums)?),
                    "Ks" => set_single!("Ks", mtl.specular = parse_color(line, &line_nums)?),
                    "Ns" => set_single!("Ns", mtl.spec_pow = parse_scalar(line, &line_nums)?),
                    "d" => set_single!("d or Tr", mtl.alpha = parse_scalar(line, &line_nums)?),
                    "Tr" => set_single!("d or Tr", mtl.alpha = 1.0 - parse_scalar(line, &line_nums)?),
                    "map_Kd" => warn!("texture map {directive} {line} is not yet supported"),
                    "map_Ka" => warn!("texture map {directive} {line} is not yet supported"),
                    "map_Ks" => warn!("texture map {directive} {line} is not yet supported"),
                    "map_Ns" => warn!("texture map {directive} {line} is not yet supported"),
                    "map_d" => warn!("texture map {directive} {line} is not yet supported"),
                    "bump" | "map_bump" => warn!("texture map {directive} {line} is not yet supported"),
                    other => debug!("unknown or unsupported directive {other} in material {mtl}; skipping."),
                }
            } else {
                // If we don't have a current name but the directive isn't `newmtl`, that's bad!!
                return Err(MtlLoadError::BeforeName(line_nums));
            }
        }

        // If we run out of lines, we are finished with our current (and last) material. Otherwise, our `continue` from
        // inside the match would have triggered. So (unless we are skipping this last one) we push what we've found so
        // far into the map.
        if let (Some(name), false) = (cur_name, skip_mode) {
            parsed_materials.push(material);
            material_indices.insert(name, parsed_materials.len() - 1);
        }

        break;
    }

    Ok(())
}

fn parse_color(line: &str, lines: &LineRange) -> MtlResult<Vec3> {
    // Line is known to be trimmed
    if line.starts_with("xyz") {
        Err(MtlLoadError::UnsupportedColorFormat(lines.clone(), "xyz"))
    } else if line.starts_with("spectral") {
        Err(MtlLoadError::UnsupportedColorFormat(lines.clone(), "spectral"))
    } else {
        let Ok((floats, remaining)) = read_ws_verts::<3>(line) else {
            return Err(MtlLoadError::InvalidColor(lines.clone()));
        };

        match floats.len() {
            1 => Ok([floats[0], floats[0], floats[0]].into()),
            3 if remaining == 0 => Ok([floats[0], floats[1], floats[2]].into()),
            _ => Err(MtlLoadError::InvalidColor(lines.clone())),
        }
    }
}

fn parse_scalar(line: &str, lines: &LineRange) -> MtlResult<f32> {
    let mut pieces = line.split_whitespace();
    match pieces.by_ref().next().map(|s| s.parse()) {
        Some(Ok(n)) if pieces.count() == 0 => Ok(n), // pieces.count() is the remaining number of bits on the line
        _ => Err(MtlLoadError::InvalidScalar(lines.clone())),
    }
}
