use std::rc::Rc;
use hexasphere::shapes::IcoSphere;

use crate::engine::Engine;
use crate::material::Material;
use crate::mesh::Mesh;

impl Mesh {
    pub fn new_procedural_terrain(size: f32, nb_subdivisions: u32, height_fn: &dyn Fn(f32, f32) -> f32, max_height: f32, engine: &mut Engine) -> Mesh {
        let mut positions = vec!([0.0, 0.0, 0.0]; (nb_subdivisions * nb_subdivisions) as usize);
        let mut indices = vec!(0; (6 * (nb_subdivisions - 1) * (nb_subdivisions - 1)) as usize);

        for x in 0..nb_subdivisions {
            for y in 0..nb_subdivisions {
                let actual_x = (x as f32 - (nb_subdivisions as f32 / 2.0)) * size / nb_subdivisions as f32;
                let actual_y = (y as f32 - (nb_subdivisions as f32 / 2.0)) * size / nb_subdivisions as f32;

                positions[(x * nb_subdivisions + y) as usize] = [actual_x, height_fn(actual_x, actual_y), actual_y];

                if x == nb_subdivisions - 1 || y == nb_subdivisions - 1 { continue; }

                let index = (6 * (x * (nb_subdivisions - 1) + y)) as usize;
                indices[index] = (x + 1) * nb_subdivisions + y;
                indices[index + 1] = x * nb_subdivisions + y;
                indices[index + 2] = x * nb_subdivisions + y + 1;
                indices[index + 3] = (x + 1) * nb_subdivisions + y;
                indices[index + 4] = x * nb_subdivisions + y + 1;
                indices[index + 5] = (x + 1) * nb_subdivisions + y + 1;
            }
        }

        let mut mesh = Mesh::from_vertex_data(indices, positions, None, engine);
        mesh.material = Rc::from(Material::new_terrain(max_height, engine));

        mesh
    }

    pub fn new_procedural_plane(size: f32, nb_subdivisions: u32, engine: &mut Engine) -> Mesh {
        Mesh::new_procedural_terrain(size, nb_subdivisions, &|_, _| 0.0, 1.0, engine)
    }

    pub fn new_procedural_sphere(diameter: f32, nb_subdivisions: u32, engine: &mut Engine) -> Mesh {
        let sphere = IcoSphere::new(nb_subdivisions as usize, |_| ());
        let vertices_raw = sphere.raw_points();
        let mut vertices: Vec<[f32;3]> = Vec::with_capacity(vertices_raw.len());

        for vertex in vertices_raw {
            vertices.push([vertex[0] * diameter / 4.0, vertex[1] * diameter / 4.0, vertex[2] * diameter / 4.0]);
        }

        let indices = sphere.get_all_indices();

        Mesh::from_vertex_data(indices, vertices, None, engine)
    }
}