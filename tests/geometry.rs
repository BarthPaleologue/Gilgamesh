use gilgamesh::geometry::cpu_mesh_buffers::CpuMeshBuffers;
use gilgamesh::geometry::{primitive, procedural};
use glam::Vec3;

fn assert_indexed_geometry(buffers: &CpuMeshBuffers) {
    let vertex_count = buffers.vertices.positions.len();
    assert!(vertex_count > 0);
    assert_eq!(buffers.vertices.colors.len(), vertex_count);
    assert_eq!(buffers.vertices.normals.len(), vertex_count);
    assert_eq!(buffers.vertices.tangents.len(), vertex_count);
    assert_eq!(buffers.vertices.uvs.len(), vertex_count);
    assert!(!buffers.indices.is_empty());
    assert_eq!(buffers.indices.len() % 3, 0);
    assert!(
        buffers
            .indices
            .iter()
            .all(|&index| (index as usize) < vertex_count),
        "triangle indices must reference existing vertices"
    );
    assert!(
        buffers
            .vertices
            .positions
            .iter()
            .flatten()
            .all(|value| value.is_finite())
    );
}

#[test]
fn primitives_produce_indexed_cpu_buffers_without_a_gpu() {
    assert_indexed_geometry(&primitive::cube());
    for resolution in [2, 3, 16, 32] {
        assert_indexed_geometry(&primitive::plane(resolution, 10.0));
        assert_indexed_geometry(&primitive::sphere(resolution));
    }
    for subdivisions in 0..=2 {
        let buffers = primitive::ico_sphere(subdivisions);
        assert_indexed_geometry(&buffers);
        assert_eq!(buffers.indices.len(), 60 * 4_usize.pow(subdivisions));
        for position in buffers.vertices.positions {
            assert!((Vec3::from_array(position).length() - 1.0).abs() < 1e-5);
        }
    }
}

#[test]
fn plane_keeps_dimensions_uvs_and_upward_triangle_winding() {
    let buffers = primitive::plane(3, 10.0);
    assert_eq!(buffers.vertices.positions.first(), Some(&[-5.0, 0.0, -5.0]));
    assert_eq!(buffers.vertices.positions.last(), Some(&[5.0, 0.0, 5.0]));
    assert_eq!(buffers.vertices.uvs.first(), Some(&[0.0, 0.0]));
    assert_eq!(buffers.vertices.uvs.last(), Some(&[1.0, 1.0]));
    for &triangle in buffers.indices.as_chunks::<3>().0 {
        let [a, b, c] =
            triangle.map(|index| Vec3::from_array(buffers.vertices.positions[index as usize]));
        assert!((b - a).cross(c - a).y > 0.0);
    }
}

#[test]
fn procedural_terrain_applies_heights_and_computes_normals() {
    let base_height = 3.125;
    let buffers = procedural::terrain(2.0, 3, &|x, z| base_height + x + 2.0 * z);
    assert_indexed_geometry(&buffers);
    let expected_normal = Vec3::new(-1.0, 1.0, -2.0).normalize();
    for (position, normal) in buffers
        .vertices
        .positions
        .iter()
        .zip(&buffers.vertices.normals)
    {
        let [x, y, z] = *position;
        assert_eq!(y, base_height + x + 2.0 * z);
        assert!((Vec3::from_array(*normal) - expected_normal).length() < 1e-5);
    }
}

#[test]
fn procedural_sphere_applies_height_along_the_radial_direction() {
    let base = procedural::sphere(6.0, 1, &|_, _, _| 0.0);
    let raised = procedural::sphere(6.0, 1, &|_, _, _| 0.5);
    assert_indexed_geometry(&raised);
    assert_eq!(base.indices, raised.indices);
    for (base, raised) in base
        .vertices
        .positions
        .iter()
        .zip(&raised.vertices.positions)
    {
        let expected = Vec3::from_array(*base) * (3.5 / 3.0);
        assert!((Vec3::from_array(*raised) - expected).length() < 1e-5);
    }
}
