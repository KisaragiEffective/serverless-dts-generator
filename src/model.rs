use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use anyhow::anyhow;
use clap::Parser;
use serde::{Deserialize, Deserializer};
use serde::de::Error;
use strum::EnumString;

#[derive(Deserialize)]
pub struct RootServerlessConfig {
    pub functions: FunctionCollection
}

#[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Function {
    pub handler: SerdeFromString<HandlerLocation>,
    #[serde(default)]
    pub events: Vec<FunctionEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandlerLocation {
    pub source_file_location: PathBuf,
    pub identifier: String,
}

impl FromStr for HandlerLocation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((src, ident)) = s.rsplit_once(".") {
            Ok(Self {
                source_file_location: PathBuf::from(src),
                identifier: ident.to_string(),
            })
        } else {
            Err(anyhow!("HandlerLocation: this handler location doesn't look like pointing to JavaScript handler."))
        }
    }
}

#[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum FunctionEvent {
    Http(FunctionHttpEventWrapper),
    Unsupported(HashMap<String, serde_yaml::Value>)
}

#[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct FunctionHttpEventWrapper {
    pub http: FunctionHttpEvent,
}

#[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum FunctionHttpEvent {
    Struct(FunctionHttpEventStruct),
}

#[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct FunctionHttpEventStruct {
    cors: bool,
    method: SerdeFromString<HttpMethod>,
    pub path: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SerdeFromString<T>(pub T);

impl<'de, T: FromStr> Deserialize<'de> for SerdeFromString<T> where T::Err: Display {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let res = T::from_str(String::deserialize(deserializer)?.as_str());
        match res {
            Ok(t) => Ok(Self(t)),
            Err(e) => Err(D::Error::custom(e))
        }
    }
}

#[derive(EnumString, Copy, Clone, Debug, Eq, PartialEq)]
#[strum(serialize_all = "UPPERCASE")]
enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum FunctionCollection {
    Paths(Vec<SimpleFunctionPath>)
}

impl FunctionCollection {
    pub fn instantiate(self, base_path: impl AsRef<Path>) -> Result<Vec<FunctionTable>, Box<dyn std::error::Error>> {
        return match self {
            FunctionCollection::Paths(paths) => {
                let mut buf = Vec::with_capacity(paths.len());
                for target in paths {
                    let target_path = target.path;
                    let path = base_path.as_ref().parent().unwrap().join(&target_path);
                    println!("processing: {path}", path = path.display());
                    if path.exists() {
                        if path.is_dir() {
                            return Err(
                                anyhow!("YAML path ({path}) points a directory", path = path.display())
                            ).map_err(Box::from)
                        } else {
                            let f = File::open(path)?;
                            let br = BufReader::new(f);
                            let f = serde_yaml::from_reader(br)?;
                            buf.push(f);
                        }
                    } else {
                        return Err(
                            anyhow!("YAML path points non-existent path: {path}", path = path.display())
                        ).map_err(Box::from)
                    }
                }
                Ok(buf)
            }
        }
    }
}

pub struct FunctionTable(pub Vec<Function>);

impl<'de> Deserialize<'de> for FunctionTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Ok(Self(HashMap::<String, Function>::deserialize(deserializer)?.into_values().collect()))
    }
}

struct SimpleFunctionPath {
    path: PathBuf,
}

//       "pattern": "\\$\\{file\\(.+\\.(yml|yaml)\\)(:[A-Za-z]+)?(\\s*,\\s*(\"[^,]*\"|'[^,]*'|[A-Za-z:._-]+)\\s*)?\\}"
impl<'de> Deserialize<'de> for SimpleFunctionPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        match String::deserialize(deserializer) {
            Ok(v) => {
                if let Some(v) = v.strip_prefix("${file(") {
                    if let Some(v) = v.strip_suffix(")}") {
                        let path = PathBuf::from_str(v).unwrap();
                        Ok(Self { path })
                    } else {
                        Err(D::Error::custom("prefix indicates its file reference, but is unclosed"))
                    }
                } else {
                    Err(D::Error::custom("this reference is required to be a file reference"))
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::SimpleFunctionPath;
    use crate::model::SimpleFunctionPath;

    #[test]
    fn can_deserialize() {
        serde_yaml::from_str::<SimpleFunctionPath>("${file(./resources/functions-auth.yml)}").expect("fail");
    }
}
