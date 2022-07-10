use std::hash::Hash;

use serde::Serialize;

/// the trait of Entity for Push
pub trait PushEntity: Serialize + 'static {
    /// the group this Entity belows
    type Group: PartialEq + Hash + 'static + Clone + Eq + Sync + Send;

    fn get_group(&self) -> &Self::Group;

    /// the Identity info of the entity
    type Identity: PartialEq + Hash + 'static + Clone + Eq + Sync + Send;

    fn get_identity(&self) -> &Self::Identity;
}

impl PushEntity for String {
    type Group = i32;

    fn get_group(&self) -> &Self::Group {
        &11
    }

    type Identity = Self;

    fn get_identity(&self) -> &Self::Identity {
        &self
    }
}
