use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PartId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaskId(pub String);

macro_rules! impl_id_from {
    ($id:ty) => {
        impl From<String> for $id {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $id {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }
    };
}

impl_id_from!(SourceId);
impl_id_from!(GroupId);
impl_id_from!(ItemId);
impl_id_from!(PartId);
impl_id_from!(TaskId);
