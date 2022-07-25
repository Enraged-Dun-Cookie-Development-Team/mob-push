pub mod android;
pub mod ios;
pub use ios::{
    ApnPush, IosBadge, IosBadgeType, IosNotify, IosPushSound, IosRichText, IosRichTextType,
};
use serde::Serializer;

pub trait NotifySerialize {
    fn serialize_field(&self) -> usize;
    fn serialize<S: Serializer>(
        &self,
        struct_serialize: &mut <S as Serializer>::SerializeStruct,
    ) -> Result<(), <S as Serializer>::Error>;
}

impl<T: NotifySerialize> NotifySerialize for Option<T> {
    fn serialize_field(&self) -> usize {
        match self {
            Some(inner) => NotifySerialize::serialize_field(inner),
            None => 0,
        }
    }

    fn serialize<S: Serializer>(
        &self,
        struct_serialize: &mut <S as Serializer>::SerializeStruct,
    ) -> Result<(), <S as Serializer>::Error> {
        match self {
            Some(inner) => NotifySerialize::serialize::<S>(inner, struct_serialize),
            None => Ok(()),
        }
    }
}
