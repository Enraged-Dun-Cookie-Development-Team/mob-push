use std::{borrow::Cow, hash::Hash};

use serde::Serialize;

/// the trait of Entity for Push
pub trait PushEntity: 'static + Sync + Send {
    /// the group this Entity belows
    type Resource: PartialEq + Hash + 'static + Clone + Eq + Sync;

    fn get_resource(&self) -> &Self::Resource;

    type Content: AsRef<str> + 'static + Sync + ?Sized;

    fn get_send_content(&self) -> &Self::Content;

    fn get_title<'s>(&'s self) -> Cow<'s, str> {
        "新饼来袭".into()
    }

    type AndroidNotify: Serialize + 'static + Sync;

    fn get_android_notify(&self) -> &Self::AndroidNotify;

    type IosNotify: Serialize + 'static + Sync;

    fn get_ios_notify(&self) -> &Self::IosNotify;
}
