// DirectX .x is an old legacy model format. Not recommended for any use, only reason it's
// here is because a lot of the old DML files were in .x format

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
    fn mesh_to_pon(&self) -> Result<Pon, String> {
        if let &DXNode::Obj { children: ref mesh_children, .. } = self {

            let verts_node = match &mesh_children[1] {
                &DXNode::Values(ref vals) => vals,
                _ => return Err("Can't find vertices for mesh".to_string())
            };
            let indices_node = match &mesh_children[3] {
                &DXNode::Values(ref vals) => vals,
                _ => return Err("Can't find indices for mesh".to_string())
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
                    _ => return Err("Can't find texcords for mesh".to_string())
                },
                _ => return Err("Can't find texcords for mesh".to_string())
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
            Ok(Pon::TypedPon(Box::new(
                TypedPon {
                    type_name: "static_mesh".to_string(),
                    data: Pon::Object(hashmap!{
                        "layout".to_string() => Pon::from_string("[['position', 3], ['texcoord', 2]]").unwrap(),
                        "vertices".to_string() => Pon::FloatArray(verts),
                        "indices".to_string() => Pon::IntegerArray(indices)
                    })
                }
                )))
        } else {
            Err("Not a mesh".to_string())
        }
    }
    fn frame_transform_to_pon(&self) -> Result<Pon, String> {
        if let &DXNode::Obj { children: ref transform_children, .. } = self {
            match &transform_children[0] {
                &DXNode::Values(ref vals) => {
                    Ok(Pon::TypedPon(Box::new(TypedPon {
                        type_name: "mul".to_string(),
                        data: Pon::Array(vec![
                            Pon::DependencyReference(NamedPropRef::new(EntityPath::Parent, "transform")),
                            Pon::TypedPon(Box::new(TypedPon {
                                type_name: "matrix".to_string(),
                                data: Pon::FloatArray(vals[0][0].clone())
                                }))
                            ])
                        })))
                },
                _ => Err("Malformed frame transform".to_string())
            }
        } else {
            Err("Not a frame transform".to_string())
        }
    }
    fn anim_from_values(target_entity: &str, property: &str, values: &Vec<Vec<Vec<f32>>>, index: usize) -> Pon {
        let keys: Vec<Vec<f32>> = values.iter().map(|v| vec![v[0][0], v[2][index]] ).collect();
        // Commented out because it hasn't been tested anywhere yet
        // // Check if all the keys are the same, in which case we can just return a fixed value instead
        // let mut all_same = true;
        // for i in 1..keys.len() {
        //     if keys[i][1] != keys[0][1] {
        //         all_same = false;
        //         break;
        //     }
        // }
        // if all_same {
        //     println!("ALL SAME");
        //     return Pon::TypedPon(Box::new(TypedPon {
        //         type_name: "fixed_value".to_string(),
        //         data: Pon::Object(hashmap!(
        //             "property" => Pon::Reference(NamedPropRef::new(EntityPath::Search(Box::new(EntityPath::This), target_entity.to_string()), property)),
        //             "value" => Pon::Float(keys[0][1])
        //         ))
        //     }))
        // }
        Pon::TypedPon(Box::new(TypedPon {
            type_name: "key_framed".to_string(),
            data: Pon::Object(hashmap!(
                "property" => Pon::Reference(NamedPropRef::new(EntityPath::Search(Box::new(EntityPath::This), target_entity.to_string()), property)),
                "loop" => Pon::String("forever".to_string()),
                "duration" => Pon::Float(keys.last().unwrap()[0]),
                "keys" => Pon::Array(keys.into_iter().map(|v| Pon::FloatArray(v) ).collect())
            ))
        }))
    }
    // anim_from_values(target_entity, "rotate_x", rotate, 0)
    fn animation_to_pon(&self) -> Result<Pon, String> {
        let children = match self {
            &DXNode::Obj { ref children, .. } => children,
            _ => unreachable!()
        };
        let target_entity = match &children[0] {
            &DXNode::Qualifier(ref target_entity) => target_entity,
            _ => unreachable!()
        };
        let rotate = {
            let children = match &children[1] {
                &DXNode::Obj { ref children, .. } => children,
                _ => unreachable!()
            };
            let values = match &children[2] {
                &DXNode::Values(ref values) => values,
                _ => unreachable!()
            };
            values
        };
        let scale = {
            let children = match &children[1] {
                &DXNode::Obj { ref children, .. } => children,
                _ => unreachable!()
            };
            let values = match &children[2] {
                &DXNode::Values(ref values) => values,
                _ => unreachable!()
            };
            values
        };
        let translate = {
            let children = match &children[1] {
                &DXNode::Obj { ref children, .. } => children,
                _ => unreachable!()
            };
            let values = match &children[2] {
                &DXNode::Values(ref values) => values,
                _ => unreachable!()
            };
            values
        };
        Ok(Pon::TypedPon(Box::new(TypedPon {
            type_name: "animation_set".to_string(),
            data: Pon::Array(vec![
                DXNode::anim_from_values(target_entity, "rotate_x", &rotate, 0),
                DXNode::anim_from_values(target_entity, "rotate_y", &rotate, 1),
                DXNode::anim_from_values(target_entity, "rotate_z", &rotate, 2),
                DXNode::anim_from_values(target_entity, "rotate_w", &rotate, 3),

                DXNode::anim_from_values(target_entity, "scale_x", &scale, 0),
                DXNode::anim_from_values(target_entity, "scale_y", &scale, 1),
                DXNode::anim_from_values(target_entity, "scale_z", &scale, 2),

                DXNode::anim_from_values(target_entity, "translate_x", &translate, 0),
                DXNode::anim_from_values(target_entity, "translate_y", &translate, 1),
                DXNode::anim_from_values(target_entity, "translate_z", &translate, 2)
            ])
        })))
    }
    fn animation_set_to_pon(&self) -> Result<Pon, String> {
        match self {
            &DXNode::Obj { ref name, ref arg, ref children } => {
                Ok(Pon::TypedPon(Box::new(TypedPon {
                    type_name: "animation_set".to_string(),
                    data: Pon::Array(children.iter().map(|c| c.animation_to_pon().unwrap()).collect())
                })))
            },
            _ => panic!("Unexpected surprise")
        }
    }
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
                        system.set_property(&ent, "diffuse".to_string(), Pon::from_string("@parent.diffuse").unwrap());
                        let transform_node = children.iter().find(|x| match x {
                            &&DXNode::Obj { ref name, .. } => name.as_str() == "FrameTransformMatrix",
                            _ => false
                        });
                        if let Some(transform_node) = transform_node {
                            system.set_property(&ent, "transform".to_string(), transform_node.frame_transform_to_pon().unwrap());
                        } else {
                            system.set_property(&ent, "transform".to_string(), Pon::from_string("@parent.transform").unwrap());
                        }
                        let mesh_node = children.iter().find(|x| match x {
                            &&DXNode::Obj { ref name, .. } => name.as_str() == "Mesh",
                            _ => false
                        });
                        if let Some(mesh) = mesh_node {
                            system.set_property(&ent, "mesh".to_string(), match mesh.mesh_to_pon() {
                                Ok(mesh) => mesh,
                                Err(err) => panic!("Failed to parse mesh for {:?}: {}", arg, err)
                            });
                        }
                        for n in children {
                            n.append_to_system(system, &ent);
                        }
                    },
                    "AnimationSet" => {
                        system.set_property(parent, format!("animation_{}", arg.clone().unwrap().to_string()), self.animation_set_to_pon().unwrap());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}
