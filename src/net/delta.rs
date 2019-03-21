// Utilities to extract state changes from two
// ECS.
//
// Fortunately, we do not send everything over the network
// At the moment, only position and render state will be
// target for the delta.
//
// For example, if the object has moved a bit, send the delta. If the mesh has morphed, send it as
// well.
use crate::ecs::{
    ECS, 
    components::{
        TransformComponent,
        ModelComponent,
    }
};
use cgmath::InnerSpace;

const EPSILON: f32 = 0.00001;

// Compute change between two ECS
//
// What kind of action:
// - UPDATE entity (if update non-existing, should create it)
// - DEALLOCATE entity
// - REMOVE ALL (for example, when changing the level)
fn compute_delta(old: &ECS, current: &ECS) {

    // Get all live entities in current
    //
    // Iterate
    //
    // If can find same entity in old, compute the difference.
    // If cannot, delete if existing but old generation and compute difference with nothing -> get current state

}

fn apply_delta(ecs: &mut ECS) {

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

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::Vector3;

    #[test]
    fn delta_transform_test() {
        let current = TransformComponent {
            position: Vector3::new(2.3, -12.0, 2.0),
            rotation: Vector3::new(0.0, 1.0, 0.0),
            scale: Vector3::new(0.0, 0.0, 0.0),
        };
        let old = TransformComponent {
            position: Vector3::new(0.0, -12.0, 0.0),
            rotation: Vector3::new(0.0, 1.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        };

        let (mpos, mrot, mscale) = compute_transform_delta(&old, &current);
        assert_eq!(Some([2.3, 0.0, 2.0]), mpos);
        assert_eq!(None, mrot);
        assert_eq!(Some([-1.0, -1.0, -1.0]), mscale);
    }

    #[test]
    fn delta_model_test() {
        let current = ModelComponent {
            mesh_name: "current".to_string(),
            texture_name: "old".to_string(),
        };

        let old = ModelComponent {
            mesh_name: "old".to_string(),
            texture_name: "old".to_string(),
        };

        let (mmesh, mtext) = compute_model_delta(&old, &current);
        assert_eq!(Some("current".to_string()), mmesh);
        assert_eq!(None, mtext);
    }
}
