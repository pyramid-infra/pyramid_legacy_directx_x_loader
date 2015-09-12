#![feature(plugin, convert)]
#![plugin(peg_syntax_ext)]
peg_file! legacy_dotx_parse("legacy_dotx.rustpeg");

#[macro_use]
extern crate pyramid;
extern crate time;
extern crate ppromise;
extern crate cgmath;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
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
use pyramid::document::*;
use ppromise::*;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::*;
use std::mem;

struct XFile {
    node: Promise<DXNode>,
    pending_scene_adds: Vec<EntityId>
}
impl XFile {
    fn new(node: Promise<DXNode>) -> XFile {
        XFile {
            node: node,
            pending_scene_adds: vec![]
        }
    }
    fn update(&mut self, system: &mut ISystem) {
        if self.pending_scene_adds.len() > 0 && self.node.value().is_some() {
            let pending_scene_adds = mem::replace(&mut self.pending_scene_adds, vec![]);
            for entity_id in pending_scene_adds {
                self.node.value().unwrap().append_to_system(system, &entity_id, 24.0);
            }
        }
    }
    fn append_to_entity(&mut self, system: &mut ISystem, entity_id: &EntityId) {
        match self.node.value().is_some() {
            true => self.node.value().unwrap().append_to_system(system, entity_id, 24.0),
            false => self.pending_scene_adds.push(*entity_id)
        }
    }
}

pub struct LegacyDotXSubSystem {
    root_path: PathBuf,
    x_files: HashMap<Pon, XFile>,
    async_runner: AsyncRunner
}

impl LegacyDotXSubSystem {
    pub fn new(root_path: PathBuf) -> LegacyDotXSubSystem {
        LegacyDotXSubSystem {
            root_path: root_path,
            x_files: HashMap::new(),
            async_runner: AsyncRunner::new_pooled(4)
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

            match self.x_files.entry(pn.clone()) {
                Entry::Occupied(o) => {
                    o.into_mut().append_to_entity(system, &pr.entity_id)
                },
                Entry::Vacant(v) => {
                    let root_path = self.root_path.clone();
                    let pon = pn.clone();
                    let xfile = XFile::new(self.async_runner.exec_async(move || dxnode_from_pon(&root_path, &pon).unwrap()));
                    v.insert(xfile).append_to_entity(system, &pr.entity_id);
                }
            }
            system.set_property(&pr.entity_id, "directx_x_loaded", pn.clone()).unwrap();
        }
    }
    fn update(&mut self, system: &mut ISystem, delta_time: time::Duration) {
        self.async_runner.try_resolve_all();
        for (_, xfile) in self.x_files.iter_mut() {
            xfile.update(system);
        }
    }
}
