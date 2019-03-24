// Utilities to extract state changes from two
// ECS.
//
// Fortunately, we do not send everything over the network
// At the moment, only position and render state will be
// target for the delta.
//
// For example, if the object has moved a bit, send the delta. If the mesh has morphed, send it as
// well.
use crate::collections::RingBuffer;
use crate::ecs::{
    components::{LightComponent, LightType, ModelComponent, TransformComponent},
    Entity, ECS,
};
use cgmath::InnerSpace;
use log::{debug, error};
use serde_derive::{Deserialize, Serialize};
const EPSILON: f32 = 0.00001;

#[derive(Debug)]
pub enum SnapshotError {
    RingBufferEmpty,
    ClientCaughtUp,
    InvalidStateIndex,
}

use std::error::Error;
use std::fmt;

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for SnapshotError {
    fn description(&self) -> &str {
        match *self {
            SnapshotError::RingBufferEmpty => "The ringbuffer is currently empty",
            SnapshotError::ClientCaughtUp => "The client's known state is too old",
            SnapshotError::InvalidStateIndex => "Provided state index is out of bound",
        }
    }
}

/// Give a delta between current snapshot and the previous state of the game.
///
/// Internally, it keeps a circular buffer with a bunch of ECS. Each clients
/// will have a last known state. The delta is computed between current and last
/// known, then sent to the client.
///
/// When a client hasn't updated its state fast enough and the circular buffer makes
/// a full round, the client will be considered disconnected. Timeout to disconnection
/// can be calculated from buffer size and frame duration. (60 fps -> 1 sec timeout =
/// buffer of size 60).
pub struct Snapshotter {
    state_buf: RingBuffer<ECS>,
    empty_ecs: ECS,
}

impl Snapshotter {
    pub fn new(ring_size: usize) -> Self {
        let state_buf = RingBuffer::new(ring_size);
        let empty_ecs = ECS::new();

        Snapshotter {
            state_buf,
            empty_ecs,
        }
    }

    /// Update ring buffer with current state.
    pub fn set_current(&mut self, ecs: &ECS) {
        // it's making a copy.
        self.state_buf.push(ECS::new_from_existing(ecs));
    }

    pub fn get_current_index(&self) -> usize {
        self.state_buf.head_index()
    }

    /// Compute snapshot between current and last known state.
    /// If return value is None. it means, we cannot compute because the
    /// last known state has been replaced by now. -> disconnect client.
    pub fn get_delta(
        &self,
        known_state: usize,
        player_entity: &Entity,
    ) -> Result<DeltaSnapshot, SnapshotError> {
        if known_state == self.state_buf.head_index() {
            return Err(SnapshotError::ClientCaughtUp);
        }

        if let Some(old_ecs) = self.state_buf.get(known_state) {
            if let Some(new_ecs) = self.state_buf.head() {
                Ok(compute_delta(old_ecs, new_ecs, player_entity))
            } else {
                Err(SnapshotError::RingBufferEmpty)
            }
        } else {
            Err(SnapshotError::InvalidStateIndex)
        }
    }

    /// From client that havn't received anything yet.
    pub fn get_full_snapshot(
        &self,
        player_entity: &Entity,
    ) -> Result<DeltaSnapshot, SnapshotError> {
        if let Some(new_ecs) = self.state_buf.head() {
            Ok(compute_delta(&self.empty_ecs, new_ecs, player_entity))
        } else {
            debug!("RingBuffer is empty? {}", self.state_buf.head_index());
            Err(SnapshotError::RingBufferEmpty)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaSnapshot {
    pub player_delta: DeltaEntity,
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

    fn empty(entity: Entity) -> DeltaEntity {
        Self {
            entity,
            delta_transform: (None, None, None),
            delta_model: (None, None),
            delta_light: (None, None),
        }
    }
}

// Compute change between two ECS
//
// What kind of action:
// - UPDATE entity (if update non-existing, should create it)
// - DEALLOCATE entity
pub fn compute_delta(old: &ECS, current: &ECS, player_entity: &Entity) -> DeltaSnapshot {
    // Did the player move? change orientation or whatever?
    let player_delta = {
        if current.is_entity_alive(player_entity) {
            let delta_transform = {
                match (
                    current.components.transforms.get(&player_entity),
                    old.components.transforms.get(&player_entity),
                ) {
                    (Some(new_transform), Some(old_transform))
                        if old.is_entity_alive(&player_entity) =>
                    {
                        compute_transform_delta(old_transform, new_transform)
                    }
                    (Some(new_transform), _) => compute_transform_delta_empty(new_transform),
                    (None, _) => (None, None, None),
                }
            };

            let delta_model = {
                match (
                    current.components.models.get(&player_entity),
                    old.components.models.get(&player_entity),
                ) {
                    (Some(new_model), Some(old_model)) if old.is_entity_alive(&player_entity) => {
                        compute_model_delta(old_model, new_model)
                    }
                    (Some(new_model), _) => compute_model_delta_empty(new_model),
                    (None, _) => (None, None),
                }
            };

            let delta_light = {
                match (
                    current.components.lights.get(&player_entity),
                    old.components.lights.get(&player_entity),
                ) {
                    (Some(new_light), Some(old_light)) => compute_light_delta(old_light, new_light),
                    (Some(new_light), None) => compute_light_delta_empty(new_light),
                    (None, _) => (None, None),
                }
            };

            DeltaEntity {
                entity: (*player_entity).clone(),
                delta_transform,
                delta_model,
                delta_light,
            }
        } else {
            DeltaEntity::empty(player_entity.clone())
        }
    };

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
        // Skip myself.
        if entity == *player_entity {
            continue;
        }

        // If can find same entity in old, compute the difference.
        let delta_transform = {
            match (
                current.components.transforms.get(&entity),
                old.components.transforms.get(&entity),
            ) {
                (Some(new_transform), Some(old_transform)) if old.is_entity_alive(&entity) => {
                    compute_transform_delta(old_transform, new_transform)
                }
                (Some(new_transform), _) => compute_transform_delta_empty(new_transform),
                (None, _) => (None, None, None),
            }
        };

        let delta_model = {
            match (
                current.components.models.get(&entity),
                old.components.models.get(&entity),
            ) {
                (Some(new_model), Some(old_model)) if old.is_entity_alive(&entity) => {
                    compute_model_delta(old_model, new_model)
                }
                (Some(new_model), _) => compute_model_delta_empty(new_model),
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
        player_delta,
        deltas,
        entities_to_delete,
    }
}

pub fn apply_delta(ecs: &mut ECS, delta_snapshot: DeltaSnapshot) {
    // First delete the entities that have to be deleted.
    for entity in &delta_snapshot.entities_to_delete {
        ecs.delete_entity(entity);
    }

    // Then apply the deltas.
    for delta in &delta_snapshot.deltas {
        // hum I wonder. Allocator should be only relevant on server side so let's just
        // override here and see if any bug :D
        if !ecs.is_entity_alive(&delta.entity) {
            ecs.overwrite(&delta.entity);

            // Maybe need to create some components.
            match &delta.delta_transform {
                (None, None, None) => (),
                _ => {
                    ecs.components
                        .transforms
                        .set(&delta.entity, TransformComponent::default());
                }
            }

            match &delta.delta_model {
                (None, None) => (),
                _ => {
                    ecs.components
                        .models
                        .set(&delta.entity, ModelComponent::default());
                }
            }

            match &delta.delta_light {
                (None, None) => (),
                _ => {
                    ecs.components
                        .lights
                        .set(&delta.entity, LightComponent::default());
                }
            }
        }

        if let Some(transform) = ecs.components.transforms.get_mut(&delta.entity) {
            apply_transform_delta(transform, &delta.delta_transform);
        }

        if let Some(model) = ecs.components.models.get_mut(&delta.entity) {
            apply_model_delta(model, &delta.delta_model);
        }

        if let Some(light) = ecs.components.lights.get_mut(&delta.entity) {
            apply_light_delta(light, &delta.delta_light);
        }
    }
}

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

fn apply_transform_delta(
    transform: &mut TransformComponent,
    delta: &(Option<[f32; 3]>, Option<[f32; 3]>, Option<[f32; 3]>),
) {
    if let Some(ref dpos) = delta.0.as_ref() {
        transform.position.x += dpos[0];
        transform.position.y += dpos[1];
        transform.position.z += dpos[2];
    }

    if let Some(ref drot) = delta.1.as_ref() {
        transform.rotation.x += drot[0];
        transform.rotation.y += drot[1];
        transform.rotation.z += drot[2];
    }

    if let Some(ref dscale) = delta.2.as_ref() {
        transform.scale.x += dscale[0];
        transform.scale.y += dscale[1];
        transform.scale.z += dscale[2];
    }
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

fn apply_model_delta(model: &mut ModelComponent, delta: &(Option<String>, Option<String>)) {
    if let Some(ref model_name) = delta.0.as_ref() {
        model.mesh_name = (*model_name).clone();
    }
    if let Some(ref texture_name) = delta.1.as_ref() {
        model.texture_name = (*texture_name).clone();
    }
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

fn apply_light_delta(light: &mut LightComponent, delta: &(Option<LightType>, Option<[f32; 3]>)) {
    if let Some(light_type) = delta.0.as_ref() {
        light.light_type = *light_type;
    }

    if let Some(color) = delta.1.as_ref() {
        light.color = *color;
    }
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
