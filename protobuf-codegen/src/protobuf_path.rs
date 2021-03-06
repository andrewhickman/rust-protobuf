use crate::ProtobufAbsolutePath;
use crate::ProtobufRelativePath;
use std::fmt;

/// Protobuf identifier can be absolute or relative.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ProtobufPath {
    Abs(ProtobufAbsolutePath),
    Rel(ProtobufRelativePath),
}

impl ProtobufPath {
    pub fn new<S: Into<String>>(path: S) -> ProtobufPath {
        let path = path.into();
        if path.starts_with('.') {
            ProtobufPath::Abs(ProtobufAbsolutePath::new(path))
        } else {
            ProtobufPath::Rel(ProtobufRelativePath::new(path))
        }
    }

    pub fn resolve(&self, package: &ProtobufAbsolutePath) -> ProtobufAbsolutePath {
        match self {
            ProtobufPath::Abs(p) => p.clone(),
            ProtobufPath::Rel(p) => {
                let mut package = package.clone();
                package.push_relative(p);
                package
            }
        }
    }
}

impl fmt::Display for ProtobufPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtobufPath::Abs(p) => write!(f, "{}", p),
            ProtobufPath::Rel(p) => write!(f, "{}", p),
        }
    }
}
