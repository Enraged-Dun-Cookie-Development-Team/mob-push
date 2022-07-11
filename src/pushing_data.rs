use std::hash::Hash;

use serde::Serialize;

/// the trait of Entity for Push
pub trait PushEntity: Serialize + 'static + Sync + Send {
    /// the group this Entity belows
    type Resource: PartialEq + Hash + 'static + Clone + Eq + Sync + Send;

    fn get_resource(&self) -> &Self::Resource;

    /// the Identity info of the entity
    type Identity: PartialEq + Hash + 'static + Clone + Eq + Sync + Send;

    fn get_identity(&self) -> &Self::Identity;
}

