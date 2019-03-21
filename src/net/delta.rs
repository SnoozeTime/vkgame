use crate::ecs::{ECS, components::{
    TransformComponent,
    ModelComponent,
}
};
use cgmath::InnerSpace;

const EPSILON: f32 = 0.00001;
// Utilities to extract state changes from two
// ECS.
//
// Fortunately, we do not send everything over the network
// At the moment, only position and render state will be
// target for the delta.
//
// For example, if the object has moved a bit, send the delta. If the mesh has morphed, send it as
// well.
fn compute_delta(old: &ECS, current: &ECS) {


}


/// Compute delta between 2 transforms. There is a tolerance parameter
/// because floating point operation.
fn compute_transform_delta(old_transform: &TransformComponent,
                           new_transform: &TransformComponent)
    -> (Option<[f32;3]>, Option<[f32;3]>, Option<[f32;3]>) {

        let delta_pos = new_transform.position - old_transform.position;
        let delta_rot = new_transform.rotation - old_transform.rotation;
        let delta_scale = new_transform.scale - old_transform.scale;

        let delta_pos = if delta_pos.magnitude2() > EPSILON {
            Some(delta_pos.into())
        } else {
            None
        };

        let delta_rot = if delta_rot.magnitude2() > EPSILON {
            Some(delta_rot.into())
        } else {
            None
        };
        let delta_scale = if delta_scale.magnitude2() > EPSILON {
            Some(delta_scale.into())
        } else {
            None
        };

        (delta_pos, delta_rot, delta_scale)
}

/// Delta between two model components is easy as the two elements are strings.
fn compute_model_delta(old_model: &ModelComponent,
                       new_model: &ModelComponent)
    -> (Option<String>, Option<String>) {

    let delta_mesh = if new_model.mesh_name != old_model.mesh_name {
        Some(new_model.mesh_name.clone())
    } else {
        None
    };
    let delta_texture = if new_model.texture_name != old_model.texture_name {
        Some(new_model.texture_name.clone())
    } else {
        None
    };

    (delta_mesh, delta_texture)
}
