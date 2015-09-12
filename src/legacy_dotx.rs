// DirectX .x is an old legacy model format. Not recommended for any use, only reason it's
// here is because a lot of the old DML files were in .x format

use pyramid::document::*;
use pyramid::pon::*;
use pyramid::interface::*;
use cgmath::*;

#[derive(PartialEq, Debug, Clone)]
pub enum DXNode {
    Obj {
        name: String,
        arg: Option<String>,
        children: Vec<DXNode>
    },
    Qualifier(String),
    Empty,
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
                                type_name: "matrix".to_string(),
                                data: Pon::FloatArray(vals[0][0].clone())
                                })))
                },
                _ => Err("Malformed frame transform".to_string())
            }
        } else {
            Err("Not a frame transform".to_string())
        }
    }
    fn anim_from_values(target_entity: &str, anim_ticks_per_second: f32, property: &str, values: &Vec<Vec<Vec<f32>>>, value_count: usize) -> Pon {
        let keys: Vec<(f32, Vec<f32>)> = values.iter().map(|v| (v[0][0] / anim_ticks_per_second, v[2][0..value_count].to_vec()) ).collect();
        // Check if all the keys are the same, in which case we can just return a fixed value instead
        let mut all_same = true;
        for i in 1..keys.len() {
            if keys[i].1 != keys[0].1 {
                all_same = false;
                break;
            }
        }
        if all_same {
            return Pon::TypedPon(Box::new(TypedPon {
                type_name: "fixed_value".to_string(),
                data: Pon::Object(hashmap!(
                    "property" => Pon::Reference(NamedPropRef::new(EntityPath::Search(Box::new(EntityPath::This), target_entity.to_string()), property)),
                    "value" => Pon::FloatArray(keys[0].1.clone())
                ))
            }))
        }
        Pon::TypedPon(Box::new(TypedPon {
            type_name: "key_framed".to_string(),
            data: Pon::Object(hashmap!(
                "property" => Pon::Reference(NamedPropRef::new(EntityPath::Search(Box::new(EntityPath::This), target_entity.to_string()), property)),
                "loop" => Pon::String("forever".to_string()),
                "duration" => Pon::Float(keys.last().unwrap().0),
                "keys" => Pon::Array(keys.into_iter().map(|v| Pon::Array(vec![Pon::Float(v.0), Pon::FloatArray(v.1)]) ).collect())
            ))
        }))
    }
    // anim_from_values(target_entity, "rotate_x", rotate, 0)
    fn animation_to_pon(&self, anim_ticks_per_second: f32) -> Result<Pon, String> {
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
            let children = match &children[2] {
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
            let children = match &children[3] {
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
            type_name: "track_set".to_string(),
            data: Pon::Array(vec![
                DXNode::anim_from_values(target_entity, anim_ticks_per_second, "rotate", &rotate, 4),
                DXNode::anim_from_values(target_entity, anim_ticks_per_second, "scale", &scale, 3),
                DXNode::anim_from_values(target_entity, anim_ticks_per_second, "translate", &translate, 3),
            ])
        })))
    }
    fn track_set_to_pon(&self, anim_ticks_per_second: f32) -> Result<Pon, String> {
        match self {
            &DXNode::Obj { ref name, ref arg, ref children } => {
                Ok(Pon::TypedPon(Box::new(TypedPon {
                    type_name: "track_set".to_string(),
                    data: Pon::Array(children.iter().map(|c| c.animation_to_pon(anim_ticks_per_second).unwrap()).collect())
                })))
            },
            _ => panic!("Unexpected surprise")
        }
    }
    pub fn append_to_system(&self, system: &mut ISystem, parent: &EntityId, mut anim_ticks_per_second: f32) {
        match self {
            &DXNode::Obj { ref name, ref arg, ref children } => {
                match name.as_str() {
                    "Root" => {
                        for n in children {
                            n.append_to_system(system, parent, anim_ticks_per_second);
                        }
                    },
                    "Frame" => {
                        let ent = system.append_entity(parent, "DXFrame".to_string(), arg.clone()).unwrap();
                        system.set_property(&ent, "diffuse", Pon::DependencyReference(NamedPropRef::new(EntityPath::Parent, "diffuse"))).unwrap();
                        let transform_node = children.iter().find(|x| match x {
                            &&DXNode::Obj { ref name, .. } => name.as_str() == "FrameTransformMatrix",
                            _ => false
                        });
                        system.set_property(&ent, "translate", Pon::Vector3(Vector3::new(0.0, 0.0, 0.0))).unwrap();
                        system.set_property(&ent, "rotate", Pon::Vector4(Vector4::new(0.0, 0.0, 0.0, 1.0))).unwrap();
                        system.set_property(&ent, "scale", Pon::Vector3(Vector3::new(1.0, 1.0, 1.0))).unwrap();
                        let mut transforms = vec![];

                        transforms.push(Pon::DependencyReference(NamedPropRef::new(EntityPath::Parent, "transform")));

                        if let Some(transform_node) = transform_node {
                            transforms.push(transform_node.frame_transform_to_pon().unwrap());
                        }
                        transforms.push(Pon::new_typed_pon("translate", Pon::DependencyReference(NamedPropRef::new(EntityPath::This, "translate"))));
                        transforms.push(Pon::new_typed_pon("rotate_quaternion", Pon::DependencyReference(NamedPropRef::new(EntityPath::This, "rotate"))));
                        transforms.push(Pon::new_typed_pon("scale", Pon::DependencyReference(NamedPropRef::new(EntityPath::This, "scale"))));

                        system.set_property(&ent, "transform", Pon::new_typed_pon("mul", Pon::Array(transforms))).unwrap();

                        system.set_property(&ent, "shader", Pon::DependencyReference(NamedPropRef::new(EntityPath::Parent, "shader"))).unwrap();
                        system.set_property(&ent, "uniforms", Pon::DependencyReference(NamedPropRef::new(EntityPath::Parent, "uniforms"))).unwrap();
                        system.set_property(&ent, "alpha", Pon::DependencyReference(NamedPropRef::new(EntityPath::Parent, "alpha"))).unwrap();

                        let mesh_node = children.iter().find(|x| match x {
                            &&DXNode::Obj { ref name, .. } => name.as_str() == "Mesh",
                            _ => false
                        });
                        if let Some(mesh) = mesh_node {
                            system.set_property(&ent, "mesh", match mesh.mesh_to_pon() {
                                Ok(mesh) => mesh,
                                Err(err) => panic!("Failed to parse mesh for {:?}: {}", arg, err)
                            }).unwrap();
                        }
                        for n in children {
                            n.append_to_system(system, &ent, anim_ticks_per_second);
                        }
                    },
                    "AnimationSet" => {
                        system.set_property(parent, &format!("animation_{}", arg.clone().unwrap().to_string()), self.track_set_to_pon(anim_ticks_per_second).unwrap()).unwrap();
                    },
                    "AnimTicksPerSecond" => {
                        anim_ticks_per_second = match children[0] {
                            DXNode::Value(v) => v,
                            _ => panic!("Didn't find anim ticks per second child")
                        };
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}
