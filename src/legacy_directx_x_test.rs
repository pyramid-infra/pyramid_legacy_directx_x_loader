peg_file! legacy_directx_x_parse("legacy_directx_x.rustpeg");
use legacy_directx_x::*;

use pyramid::document::*;
use pyramid::propnode::*;
use pyramid::interface::*;
use pyramid::system::*;

#[test]
fn test_load_file_1() {
    let data = r#"xof 0303txt 0032

        Frame polySurface1 {
        	FrameTransformMatrix {
        		1.000000,0.000000,-0.000000,0.000000,0.000000,1.000000,-0.000000,0.000000,-0.000000,-0.000000,1.000000,0.000000,0.000000,0.000000,-0.000000,1.000000;;
        	}
        	Mesh polySurfaceShape1 {
        		2;
        		-4.382881;  6.532472;  1.292141;,
        		-3.229391;  6.532472;  0.391409;;
        		2;
        		3;327,326,325;,
        		4;331,330,329,328;;
        		MeshNormals {
        			2;
        			-0.028629, 0.999570, 0.006254;,
        			-0.053262, 0.998496, 0.012994;;
        			2;
        			3;127,125,126;,
        			4;124,120,121,123;;
        		}
        		MeshTextureCoords {
        			2;
        			 0.899474;-0.274396;,
        			 0.678246;-0.182760;;
        		}
        	}
        }
    "#;
    let dxnode = legacy_directx_x_parse::file(data).unwrap();

    let mut system = System::new();
    let ent = system.append_entity(&-1, "Entity".to_string(), None).unwrap();
    dxnode.append_to_system(&mut system, &ent);
    let mesh_ent = system.get_entity_by_name("polySurface1").unwrap();
    let mesh = system.get_property_value(&mesh_ent, "mesh");
    assert_eq!(mesh, Ok(PropNode::PropTransform(Box::new(
        PropTransform {
            name: "static_mesh".to_string(),
            arg: PropNode::Object(hashmap!{
                "vertices".to_string() => PropNode::FloatArray(vec![
                        -4.382881, 6.532472, 1.292141, 0.899474, -0.274396,
                        -3.229391, 6.532472, 0.391409, 0.678246, -0.182760
                    ]),
                "indices".to_string() => PropNode::IntegerArray(vec![
                        327,326,325,
                        331,330,329,
                        331,329,328
                    ])
            })
        }
        ))));
}


#[test]
fn test_load_file_2() {
    let data = r#"xof 0303txt 0032

    Frame pCube1 {
    	FrameTransformMatrix {
    		1.000000,0.000000,-0.000000,0.000000,0.000000,1.000000,-0.000000,0.000000,-0.000000,-0.000000,1.000000,0.000000,0.000000,0.000000,-0.000000,1.000000;;
    	}
    }
    Frame pCube2 {
    	FrameTransformMatrix {
    		1.000000,0.000000,-0.000000,0.000000,-0.000000,-1.000000,0.000000,0.000000,-0.000000,-0.000000,1.000000,0.000000,0.000000,0.000000,-0.000000,1.000000;;
    	}
    }
    Frame polySurface1 {
    	FrameTransformMatrix {
    		1.000000,0.000000,-0.000000,0.000000,0.000000,1.000000,-0.000000,0.000000,-0.000000,-0.000000,1.000000,0.000000,0.000000,0.000000,-0.000000,1.000000;;
    	}
    	Mesh polySurfaceShape1 {
    		2;
    		 0.856444;  0.000000; -1.511363;,
    		 0.856444;  0.000000; -1.511363;;
    		2;
    		4;3,2,1,0;,
    		4;67,66,65,64;;
    		MeshNormals {
    			2;
    			 0.654350, 0.437199,-0.616995;,
    			 0.061550, 0.425390,-0.902915;;
    			2;
    			4;15,14,26,13;,
    			4;26,27,12,13;;
    		}
    		MeshTextureCoords {
    			2;
    			 0.706463;-0.177258;,
    			 0.706463;-0.177258;;
    		}
    	}
    }
"#;
    let dxnode = legacy_directx_x_parse::file(data).unwrap();

    let mut system = System::new();
    let ent = system.append_entity(&-1, "Entity".to_string(), None).unwrap();
    dxnode.append_to_system(&mut system, &ent);
    let mesh_ent = system.get_entity_by_name("polySurface1").unwrap();
    let mesh = system.get_property_value(&mesh_ent, "mesh");
    assert_eq!(mesh, Ok(PropNode::PropTransform(Box::new(
        PropTransform {
            name: "static_mesh".to_string(),
            arg: PropNode::Object(hashmap!{
                "vertices".to_string() => PropNode::FloatArray(vec![
                    0.856444, 0.000000, -1.511363, 0.706463, -0.177258,
                    0.856444, 0.000000, -1.511363, 0.706463, -0.177258
                    ]),
                "indices".to_string() => PropNode::IntegerArray(vec![
                    3,2,1,
                    3,1,0,
                    67,66,65,
                    67,65,64
                    ])
            })
        }
        ))));
}


#[test]
fn test_load_file_3() {
    let data = r#"xof 0303txt 0032

	AnimationKey {
		1;4;-0.027192,-0.792217,-0.093974;;,
		110;4;-0.027192,-0.792217,-0.093974;;;
	}
"#;
    let parsed = legacy_directx_x_parse::file(data);
    parsed.unwrap();
    //assert!(parsed.is_ok());
}
