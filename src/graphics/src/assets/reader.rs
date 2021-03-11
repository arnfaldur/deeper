use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use itertools::Itertools;
use serde::de::DeserializeOwned;
use wavefront_obj::obj;

use crate::data;

pub fn read_ron<T: DeserializeOwned>(path: &Path) -> Result<T, ron::Error> {
    let data = fs::read_to_string(path)?;
    ron::de::from_bytes(data.as_bytes())
}

// TODO: Handle transforms
pub fn vertex_lists_from_gltf(path: &Path) -> Result<data::VertexLists, String> {
    let (document, buffers, _images) = gltf::import(path).expect(
        format!(
            "[graphics/gltf] : File {} could not be opened",
            path.display()
        )
        .as_ref(),
    );

    // TODO: Add checks for multiple models/scenes, etc.
    let mut vertex_lists = vec![];

    for mesh in document.meshes() {
        let mut vertex_list = vec![];
        for primitive in mesh.primitives() {
            // TODO: Is there a more readable way to do this
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // TODO: This feels ... wrong
            let positions = reader.read_positions().unwrap().collect_vec();
            let normals = reader.read_normals().unwrap().collect_vec();
            // TODO: What is set?
            let tex_coords = reader.read_tex_coords(0).unwrap().into_f32().collect_vec();

            let indices = reader.read_indices().unwrap().into_u32();

            for idx in indices {
                let pos = positions.get(idx as usize).unwrap().clone();
                let normal = normals.get(idx as usize).unwrap().clone();
                let tex_coord = tex_coords.get(idx as usize).unwrap().clone();

                vertex_list.push(data::Vertex {
                    pos,
                    normal,
                    tex_coord,
                })
            }
        }
        vertex_lists.push(vertex_list);
    }

    return Ok(vertex_lists);
}

pub fn vertex_lists_from_obj(path: &Path) -> Result<data::VertexLists, String> {
    let mut f;

    if let Ok(file) = File::open(path) {
        f = file;
    } else {
        return Err(format!(
            "[graphics] : File {} could not be opened.",
            path.display()
        ));
    };

    let mut buf = String::new();
    let _ = f.read_to_string(&mut buf);

    let obj_set = obj::parse(buf).expect("Failed to parse obj file");

    let mut vertex_lists = vec![];

    for obj in &obj_set.objects {
        let mut vertices = vec![];

        for geometry in &obj.geometry {
            let mut indices = vec![];

            geometry.shapes.iter().for_each(|shape| {
                if let obj::Primitive::Triangle(v1, v2, v3) = shape.primitive {
                    indices.push(v1);
                    indices.push(v2);
                    indices.push(v3);
                }
            });

            for idx in &indices {
                let pos = obj.vertices[idx.0];

                let normal = match idx.2 {
                    Some(i) => obj.normals[i],
                    _ => obj::Normal {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                };

                let tc = match idx.1 {
                    Some(i) => obj.tex_vertices[i],
                    _ => obj::TVertex {
                        u: 0.0,
                        v: 0.0,
                        w: 0.0,
                    },
                };

                let v = data::Vertex {
                    pos: [pos.x as f32, pos.y as f32, pos.z as f32],
                    normal: [normal.x as f32, normal.y as f32, normal.z as f32],
                    tex_coord: [tc.u as f32, tc.v as f32],
                };
                vertices.push(v);
            }
        }
        vertex_lists.push(vertices);
    }
    Ok(vertex_lists)
}
