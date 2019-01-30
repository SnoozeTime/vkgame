use cgmath::{Vector3};
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(remote = "Vector3")]
pub struct VectorDef<S> {
    x: S,
    y: S,
    z: S,
}
