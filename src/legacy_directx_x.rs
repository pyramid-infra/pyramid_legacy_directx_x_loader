// DirectX .x is an old legacy model format. Not recommended for any use, only reason it's
// here is because a lot of the old DML files were in .x format

use pyramid::*;
use pyramid::document::*;
use pyramid::propnode::*;
use pyramid::interface::*;

#[derive(PartialEq, Debug, Clone)]
pub enum DXNode {
    Obj {
        name: String,
        arg: Option<String>,
        children: Vec<DXNode>
    },
    Qualifier(String),
    Value(f32),
    Values(Vec<Vec<Vec<f32>>>)
}

impl DXNode {
    pub fn append_to_system(&self, system: &mut ISystem, parent: &EntityId) {
        println!("APPEND {:?}", self);
        match self {
            &DXNode::Obj { ref name, ref arg, ref children } => {
                match name.as_str() {
                    "Root" => {
                        for n in children {
                            n.append_to_system(system, parent);
                        }
                    },
                    "Frame" => {
                        let ent = system.append_entity(parent, "DXFrame".to_string(), arg.clone()).unwrap();
                        let mesh_node = children.iter().find(|x| match x {
                            &&DXNode::Obj { ref name, .. } => name.as_str() == "Mesh",
                            _ => false
                        });
                        if let Some(&DXNode::Obj { children: ref mesh_children, .. }) = mesh_node {

                            let verts_node = match &mesh_children[1] {
                                &DXNode::Values(ref vals) => vals,
                                _ => panic!("Can't find vertices for mesh {:?}", arg)
                            };
                            let indices_node = match &mesh_children[3] {
                                &DXNode::Values(ref vals) => vals,
                                _ => panic!("Can't find indices for mesh {:?}", arg)
                            };
                            let texcords_node = match mesh_children.iter().find(|x| {
                                if let &&DXNode::Obj { ref name, .. } = x {
                                    if name == "MeshTextureCoords" {
                                        return true;
                                    }
                                }
                                false
                            }) {
                                Some(&DXNode::Obj { ref children, .. }) => match &children[1] {
                                    &DXNode::Values(ref values) => values,
                                    _ => panic!("Can't find texcords for mesh {:?}", arg)
                                },
                                _ => panic!("Can't find texcords for mesh {:?}", arg)
                            };
                            let mut verts = vec![];
                            for i in 0..verts_node.len() {
                                verts.push(verts_node[i][0][0]);
                                verts.push(verts_node[i][1][0]);
                                verts.push(verts_node[i][2][0]);
                                verts.push(texcords_node[i][0][0]);
                                verts.push(texcords_node[i][1][0]);
                            }
                            let mut indices = vec![];
                            for inds in indices_node {
                                if inds[1].len() == 4 {
                                    indices.push(inds[1][0] as i64);
                                    indices.push(inds[1][1] as i64);
                                    indices.push(inds[1][2] as i64);
                                    indices.push(inds[1][0] as i64);
                                    indices.push(inds[1][2] as i64);
                                    indices.push(inds[1][3] as i64);
                                } else if inds[1].len() == 3 {
                                    indices.push(inds[1][0] as i64);
                                    indices.push(inds[1][1] as i64);
                                    indices.push(inds[1][2] as i64);
                                }
                            }
                            system.set_property(&ent, "mesh".to_string(), PropNode::PropTransform(Box::new(
                                PropTransform {
                                    name: "static_mesh".to_string(),
                                    arg: PropNode::Object(hashmap!{
                                        "vertices".to_string() => PropNode::FloatArray(verts),
                                        "indices".to_string() => PropNode::IntegerArray(indices)
                                    })
                                }
                                )))
                        }
                        for n in children {
                            n.append_to_system(system, &ent);
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
    // fn get_mesh_node(&self, id: &String) -> Option<&DXNode> {
    //     match self {
    //         &DXNode::Obj { ref name, ref arg, ref children } => {
    //             if let &Some(ref arg) = arg {
    //                 if name == "Mesh" && *arg == *id {
    //                     return Some(self);
    //                 }
    //             }
    //             for k in children {
    //                 if let Some(n) = k.get_mesh_node(id) {
    //                     return Some(n);
    //                 }
    //             }
    //             None
    //         },
    //         _ => None
    //     }
    // }
    // pub fn to_mesh(&self, id: String) -> Result<Mesh, String> {
    //     let node_children = match self.get_mesh_node(&id) {
    //         Some(&DXNode::Obj { ref children, .. }) => children,
    //         _ => return Err(format!("Can't find mesh node: {}", id))
    //     };
    //     let verts_node = match &node_children[1] {
    //         &DXNode::Values(ref vals) => vals,
    //         _ => return Err(format!("Can't find vertices for mesh {}", id))
    //     };
    //     let indices_node = match &node_children[3] {
    //         &DXNode::Values(ref vals) => vals,
    //         _ => return Err(format!("Can't find indices for mesh {}", id))
    //     };
    //     let texcords_node = match node_children.iter().find(|x| {
    //         if let &&DXNode::Obj { ref name, .. } = x {
    //             if name == "MeshTextureCoords" {
    //                 return true;
    //             }
    //         }
    //         false
    //     }) {
    //         Some(&DXNode::Obj { ref children, .. }) => match &children[1] {
    //             &DXNode::Values(ref values) => values,
    //             _ => return Err(format!("Can't find texcords for mesh {}", id))
    //         },
    //         _ => return Err(format!("Can't find texcords for mesh {}", id))
    //     };
    //     let mut verts = vec![];
    //     for i in 0..verts_node.len() {
    //         verts.push(verts_node[i][0][0]);
    //         verts.push(verts_node[i][1][0]);
    //         verts.push(verts_node[i][2][0]);
    //         verts.push(texcords_node[i][0][0]);
    //         verts.push(texcords_node[i][1][0]);
    //     }
    //     let mut indices = vec![];
    //     for inds in indices_node {
    //         if inds[1].len() == 4 {
    //             indices.push(inds[1][0] as u32);
    //             indices.push(inds[1][1] as u32);
    //             indices.push(inds[1][2] as u32);
    //             indices.push(inds[1][0] as u32);
    //             indices.push(inds[1][2] as u32);
    //             indices.push(inds[1][3] as u32);
    //         } else if inds[1].len() == 3 {
    //             indices.push(inds[1][0] as u32);
    //             indices.push(inds[1][1] as u32);
    //             indices.push(inds[1][2] as u32);
    //         }
    //     }
    //     return Ok(Mesh {
    //         vertices: verts,
    //         indices: indices
    //     });
    // }
}
