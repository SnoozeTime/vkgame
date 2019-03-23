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
    components::{LightComponent, LightType, ModelComponent, TransformComponent},
    Entity, ECS,
};
use cgmath::InnerSpace;
use serde_derive::{Deserialize, Serialize};

const EPSILON: f32 = 0.00001;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaSnapshot {
    pub deltas: Vec<DeltaEntity>,
    pub entities_to_delete: Vec<Entity>,
}

// That is the change for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaEntity {
    pub entity: Entity,
    //TODO is using 0 instead of option better? (smaller packet size)
    pub delta_transform: (Option<[f32; 3]>, Option<[f32; 3]>, Option<[f32; 3]>),
    pub delta_model: (Option<String>, Option<String>),
    pub delta_light: (Option<LightType>, Option<[f32; 3]>),
}

impl DeltaEntity {
    fn is_empty(&self) -> bool {
        match self.delta_transform {
            (None, None, None) => (),
            _ => return false,
        }

        match self.delta_model {
            (None, None) => (),
            _ => return false,
        }

        match self.delta_light {
            (None, None) => (),
            _ => return false,
        }

        true
    }
}

// Compute change between two ECS
//
// What kind of action:
// - UPDATE entity (if update non-existing, should create it)
// - DEALLOCATE entity
pub fn compute_delta(old: &ECS, current: &ECS) -> DeltaSnapshot {
    // Deallocating should be done first on client side to remove
    // outdated entities.
    // Find entities to delete, i.e. alive before but dead now.
    let mut entities_to_delete: Vec<_> = old
        .nb_entities()
        .iter()
        .filter(|entity| !current.is_entity_alive(&entity))
        .map(|e| *e)
        .collect();

    // Get all live entities in current
    let mut deltas = Vec::new();
    for entity in current.nb_entities() {
        // If can find same entity in old, compute the difference.
        let delta_transform = {
            match (
                current.components.transforms.get(&entity),
                old.components.transforms.get(&entity),
            ) {
                (Some(new_transform), Some(old_transform)) => {
                    compute_transform_delta(old_transform, new_transform)
                }
                (Some(new_transform), None) => compute_transform_delta_empty(new_transform),
                (None, _) => (None, None, None),
            }
        };

        let delta_model = {
            match (
                current.components.models.get(&entity),
                old.components.models.get(&entity),
            ) {
                (Some(new_model), Some(old_model)) => compute_model_delta(old_model, new_model),
                (Some(new_model), None) => compute_model_delta_empty(new_model),
                (None, _) => (None, None),
            }
        };

        let delta_light = {
            match (
                current.components.lights.get(&entity),
                old.components.lights.get(&entity),
            ) {
                (Some(new_light), Some(old_light)) => compute_light_delta(old_light, new_light),
                (Some(new_light), None) => compute_light_delta_empty(new_light),
                (None, _) => (None, None),
            }
        };

        let delta_entity = DeltaEntity {
            entity,
            delta_transform,
            delta_model,
            delta_light,
        };

        if !delta_entity.is_empty() {
            deltas.push(delta_entity);
        }
    }

    DeltaSnapshot {
        deltas,
        entities_to_delete,
    }
}

fn apply_delta(ecs: &mut ECS) {}

/* ----------------------------------------------------------------------------------
 * Components delta. Maybe should implement that as macro or in the component.rs file...
 *----------------------------------------------------------------------------------*/

/// Compute delta between 2 transforms. There is a tolerance parameter
/// because floating point operation.
fn compute_transform_delta(
    old_transform: &TransformComponent,
    new_transform: &TransformComponent,
) -> (Option<[f32; 3]>, Option<[f32; 3]>, Option<[f32; 3]>) {
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

/// Same as above with empty old_transform :D
fn compute_transform_delta_empty(
    new_transform: &TransformComponent,
) -> (Option<[f32; 3]>, Option<[f32; 3]>, Option<[f32; 3]>) {
    (
        Some(new_transform.position.into()),
        Some(new_transform.rotation.into()),
        Some(new_transform.scale.into()),
    )
}

/// Delta between two model components is easy as the two elements are strings.
fn compute_model_delta(
    old_model: &ModelComponent,
    new_model: &ModelComponent,
) -> (Option<String>, Option<String>) {
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

fn compute_model_delta_empty(new_model: &ModelComponent) -> (Option<String>, Option<String>) {
    (
        Some(new_model.mesh_name.clone()),
        Some(new_model.texture_name.clone()),
    )
}

fn compute_light_delta(
    old_light: &LightComponent,
    new_light: &LightComponent,
) -> (Option<LightType>, Option<[f32; 3]>) {
    let lt = if old_light.light_type == new_light.light_type {
        None
    } else {
        Some(new_light.light_type)
    };

    let color_diff = [
        new_light.color[0] - old_light.color[0],
        new_light.color[1] - old_light.color[1],
        new_light.color[2] - old_light.color[2],
    ];

    let color = if color_diff[0] * color_diff[0]
        + color_diff[0] * color_diff[0]
        + color_diff[0] * color_diff[0]
        < EPSILON
    {
        None
    } else {
        Some(color_diff)
    };

    (lt, color)
}

fn compute_light_delta_empty(new_light: &LightComponent) -> (Option<LightType>, Option<[f32; 3]>) {
    (Some(new_light.light_type), Some(new_light.color))
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
