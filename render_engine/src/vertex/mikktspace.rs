// For GLTF, the normal map needs to be converted with the tangents in mikktspace.
// https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#foreword:~:text=When%20tangents%20are%20not%20specified%2C%20client%20implementations%20SHOULD%20calculate%20tangents
//
// Its need is explained on http://www.mikktspace.com/
//
// The Bevy community provides a no-dependency pure rust implementation; https://docs.rs/bevy_mikktspace/latest/bevy_mikktspace/index.html
// Example use here: https://github.com/bevyengine/bevy/blob/98a5aa50c3f0ba3af72920b994387790230f43d9/crates/bevy_mesh/src/mikktspace.rs#L1
//
// This file provides an implementation of that trait for my Mesh struct, such that we can have tangents.

use glam::vec4;

/// Map from the split face/vert situation to the index, this is the index into the indices.
fn face_vert_to_index(face: usize, vert: usize) -> usize {
    face * 3 + vert
}

impl bevy_mikktspace::Geometry for super::mesh::CpuMesh {
    fn num_faces(&self) -> usize {
        self.index.len() / 3
    }

    fn num_vertices_of_face(&self, face: usize) -> usize {
        let _ = face;
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        let index = face_vert_to_index(face, vert);
        let vertex_index = self.index[index] as usize;
        self.position[vertex_index].into()
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        let index = face_vert_to_index(face, vert);
        let vertex_index = self.index[index] as usize;
        self.normal.as_ref().unwrap()[vertex_index].into()
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        let index = face_vert_to_index(face, vert);
        let vertex_index = self.index[index] as usize;
        self.uv.as_ref().unwrap()[vertex_index].into()
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        let index = face_vert_to_index(face, vert);
        let vertex_index = self.index[index] as usize;

        // https://github.com/bevyengine/bevy/blob/98a5aa50c3f0ba3af72920b994387790230f43d9/crates/bevy_mesh/src/mikktspace.rs#L126-L129
        // Bevy does; tangent[3] = -tangent[3];
        // to ensure its in a RH coordinate system, so lets do the same.
        let tangent = vec4(tangent[0], tangent[1], tangent[2], -tangent[3]);

        self.tangents.as_mut().unwrap()[vertex_index] = tangent;
    }
}
