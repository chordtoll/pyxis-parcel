/// An error enum for return from parcel methods that may fail
#[derive(Debug)]
pub enum ParcelError {
    IO(std::io::Error),
    Yaml(serde_yaml::Error),
    StringConversion,
    Enoent,
    NotFile,
}

impl From<std::io::Error> for ParcelError {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<serde_yaml::Error> for ParcelError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::Yaml(e)
    }
}
