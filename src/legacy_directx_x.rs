// DirectX .x is an old legacy model format. Not recommended for any use, only reason it's
// here is because a lot of the old DML files were in .x format

use pyramid::propnode_parser as propnode_parser;
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
                                )));
                            system.set_property(&ent, "transform".to_string(), propnode_parser::parse("@parent.transform").unwrap());
                            system.set_property(&ent, "texture".to_string(), propnode_parser::parse("@parent.texture").unwrap());
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
}
