use std::hash::Hash;

use serde::Serialize;

/// the trait of Entity for Push
pub trait PushEntity: Serialize + 'static + Sync + Send {
    /// the group this Entity belows
    type Resource: PartialEq + Hash + 'static + Clone + Eq + Sync;

    fn get_resource(&self) -> &Self::Resource;

    type AndroidNotify: Serialize + 'static + Sync;

    fn get_android_notify(&self) -> &Self::AndroidNotify;

    type IosNotify: Serialize + 'static + Sync;

    fn get_ios_notify(&self) -> &Self::IosNotify;
}
