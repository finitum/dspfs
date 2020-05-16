use serde::de::Deserialize;
use serde::ser::Serialize;

pub trait Message: Serialize + for<'de> Deserialize<'de> {}
