// DirectX .x is an old legacy model format. Not recommended for any use, only reason it's
// here is because a lot of the old DML files were in .x format

use pyramid::pon_parser as pon_parser;
use pyramid::document::*;
use pyramid::pon::*;
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
                        system.set_property(&ent, "diffuse".to_string(), pon_parser::parse("@parent.diffuse").unwrap());
                        let transform_node = children.iter().find(|x| match x {
                            &&DXNode::Obj { ref name, .. } => name.as_str() == "FrameTransformMatrix",
                            _ => false
                        });
                        if let Some(&DXNode::Obj { children: ref transform_children, .. }) = transform_node {
                            match &transform_children[0] {
                                &DXNode::Values(ref vals) => {
                                    system.set_property(&ent, "transform".to_string(), Pon::PropTransform(Box::new(PropTransform {
                                        name: "mul".to_string(),
                                        arg: Pon::Array(vec![
                                            Pon::DependencyReference(NamedPropRef { entity_name: "parent".to_string(), property_key: "transform".to_string() }),
                                            Pon::PropTransform(Box::new(PropTransform {
                                                name: "matrix".to_string(),
                                                arg: Pon::FloatArray(vals[0][0].clone())
                                                }))
                                            ])
                                        })));
                                },
                                _ => {}
                            }
                        } else {
                            system.set_property(&ent, "transform".to_string(), pon_parser::parse("@parent.transform").unwrap());
                        }
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
                            let texcoords_node = match mesh_children.iter().find(|x| {
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
                                verts.push(texcoords_node[i][0][0]);
                                verts.push(texcoords_node[i][1][0]);
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
                            system.set_property(&ent, "mesh".to_string(), Pon::PropTransform(Box::new(
                                PropTransform {
                                    name: "static_mesh".to_string(),
                                    arg: Pon::Object(hashmap!{
                                        "layout".to_string() => pon_parser::parse("[['position', 3], ['texcoord', 2]]").unwrap(),
                                        "vertices".to_string() => Pon::FloatArray(verts),
                                        "indices".to_string() => Pon::IntegerArray(indices)
                                    })
                                }
                                )));
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
