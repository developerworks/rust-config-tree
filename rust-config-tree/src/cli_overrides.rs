//! Runtime provider for CLI field override values.

use figment::{
    Metadata, Profile, Provider,
    value::{Dict, Map, Value},
};
use serde::Serialize;

use crate::config::ConfigResult;

/// Sparse override provider built from CLI fields.
#[derive(Debug, Clone, Default)]
pub struct ConfigOverrideProvider {
    values: Dict,
}

impl ConfigOverrideProvider {
    /// Creates an empty override provider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether this provider has no override values.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Inserts one override value at a dotted config path.
    pub fn insert<T>(&mut self, path: &str, value: &T) -> ConfigResult<()>
    where
        T: Serialize + ?Sized,
    {
        if path.is_empty() || path.split('.').any(str::is_empty) {
            return Err(figment::Error::from(format!(
                "config override path `{path}` must not be empty"
            ))
            .into());
        }

        let value = Value::serialize(value)?;
        let nested = figment::util::nest(path, value)
            .into_dict()
            .ok_or_else(|| {
                figment::Error::from(format!(
                    "config override path `{path}` must produce a dictionary"
                ))
            })?;
        merge_dict(&mut self.values, nested);
        Ok(())
    }
}

impl Provider for ConfigOverrideProvider {
    fn metadata(&self) -> Metadata {
        Metadata::named("CLI overrides")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        Ok(Profile::Default.collect(self.values.clone()))
    }
}

/// Builds config override values from parsed CLI input.
pub trait ConfigOverrides {
    /// Builds an override provider that can be merged into Figment.
    fn config_overrides(&self) -> ConfigResult<ConfigOverrideProvider>;
}

fn merge_dict(target: &mut Dict, source: Dict) {
    for (key, value) in source {
        match (target.remove(&key), value) {
            (Some(Value::Dict(tag, mut target_child)), Value::Dict(_, source_child)) => {
                merge_dict(&mut target_child, source_child);
                target.insert(key, Value::Dict(tag, target_child));
            }
            (_, value) => {
                target.insert(key, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_nests_override_values_by_dot_path() {
        let mut provider = ConfigOverrideProvider::new();
        provider.insert("server.port", &9000u16).unwrap();
        provider.insert("log.level", "debug").unwrap();

        let data = provider.data().unwrap();
        let values = data.get(&Profile::Default).unwrap();

        assert_eq!(
            values
                .get("server")
                .unwrap()
                .find_ref("port")
                .unwrap()
                .to_u128(),
            Some(9000)
        );
        assert_eq!(
            values
                .get("log")
                .unwrap()
                .find_ref("level")
                .unwrap()
                .as_str(),
            Some("debug")
        );
    }

    #[test]
    fn provider_merges_sibling_values_under_same_parent() {
        let mut provider = ConfigOverrideProvider::new();
        provider.insert("server.bind", "0.0.0.0").unwrap();
        provider.insert("server.port", &9000u16).unwrap();

        let data = provider.data().unwrap();
        let server = data.get(&Profile::Default).unwrap().get("server").unwrap();

        assert_eq!(server.find_ref("bind").unwrap().as_str(), Some("0.0.0.0"));
        assert_eq!(server.find_ref("port").unwrap().to_u128(), Some(9000));
    }

    #[test]
    fn provider_rejects_empty_override_path_segments() {
        let mut provider = ConfigOverrideProvider::new();

        assert!(provider.insert("", "debug").is_err());
        assert!(provider.insert("log..level", "debug").is_err());
    }
}
