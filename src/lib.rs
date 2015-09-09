#![feature(plugin, convert)]
#![plugin(peg_syntax_ext)]
peg_file! legacy_dotx_parse("legacy_dotx.rustpeg");

#[macro_use]
extern crate pyramid;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::error::Error;
use std::io::prelude::*;

mod legacy_dotx;
mod legacy_dotx_test;

use legacy_dotx::*;

use pyramid::interface::*;
use pyramid::pon::*;

pub struct LegacyDotXSubSystem {
    root_path: PathBuf,
    x_files: HashMap<Pon, DXNode>
}

impl LegacyDotXSubSystem {
    pub fn new(root_path: PathBuf) -> LegacyDotXSubSystem {
        LegacyDotXSubSystem {
            root_path: root_path,
            x_files: HashMap::new()
        }
    }
}

fn dxnode_from_pon(root_path: &PathBuf, pon: &Pon) -> Result<DXNode, PonTranslateErr> {
    let filename = try!(pon.translate::<&str>());
    let path_buff = root_path.join(Path::new(filename));
    let path = path_buff.as_path();
    println!("Loading .x file {:?}", path);
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", filename, Error::description(&why)),
        Ok(file) => file,
    };
    let mut content = String::new();
    return match file.read_to_string(&mut content) {
        Ok(_) => {
            let dx = match legacy_dotx_parse::file(&content.as_str()) {
                Ok(mesh) => mesh,
                Err(err) => panic!("Failed to load .x {:?} with error: {:?}", path, err)
            };
            println!("Loaded .x {}", filename);
            Ok(dx)
        },
        Err(err) => Err(PonTranslateErr::Generic(format!("Failed to load .x: {:?}: {:?}", path, err)))
    }
}

impl ISubSystem for LegacyDotXSubSystem {
    fn on_property_value_change(&mut self, system: &mut ISystem, prop_refs: &Vec<PropRef>) {
        for pr in prop_refs.iter().filter(|pr| pr.property_key == "directx_x") {
            let pn = system.get_property_value(&pr.entity_id, &pr.property_key.as_str()).unwrap().clone();
            match system.get_property_value(&pr.entity_id, "directx_x_loaded") {
                Ok(_) => {
                    println!("WARNING: Trying to change .x file on entity that's already been assigned a .x file once {:?}, skipping.", pr);
                    continue;
                },
                Err(_) => {}
            }

            let dx = match self.x_files.get(&pn) {
                Some(dx) => Some(dx.clone()),
                None => None
            };
            let dx = match &dx {
                &Some(ref dx) => dx,
                &None => {
                    let dx = dxnode_from_pon(&self.root_path, &pn).unwrap();
                    self.x_files.insert(pn.clone(), dx);
                    self.x_files.get(&pn).unwrap()
                }
            };
            dx.append_to_system(system, &pr.entity_id, 24.0);
            system.set_property(&pr.entity_id, "directx_x_loaded", pn.clone()).unwrap();
        }
    }
}
