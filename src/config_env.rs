//! Figment environment provider backed by `confique` metadata.
//!
//! Figment's raw environment provider can split keys on delimiters. This module
//! instead reads exact `#[config(env = "...")]` names from the schema metadata
//! and maps only those variables to their real field paths.

use std::{collections::HashMap, sync::Arc};

use confique::{
    Config,
    meta::{FieldKind, Meta},
};
use figment::{
    Metadata, Profile, Provider,
    providers::Env,
    value::{Dict, Map, Uncased},
};

/// Figment provider that maps environment variables declared in `confique`
/// schema metadata onto their exact field paths.
///
/// This provider reads `#[config(env = "...")]` from [`Config::META`] and
/// avoids Figment's delimiter-based environment key splitting. Environment
/// variables such as `APP_DATABASE_POOL_SIZE` can therefore map to a Rust field
/// named `database.pool_size` without treating the single underscores as nested
/// separators.
#[derive(Clone)]
pub struct ConfiqueEnvProvider {
    env: Env,
    path_to_env: Arc<HashMap<String, String>>,
}

/// Constructors for environment providers derived from `confique` metadata.
impl ConfiqueEnvProvider {
    /// Creates an environment provider for a `confique` schema.
    ///
    /// # Type Parameters
    ///
    /// - `S`: Config schema whose metadata declares environment variable names.
    ///
    /// # Arguments
    ///
    /// This function has no arguments.
    ///
    /// # Returns
    ///
    /// Returns a provider that emits only environment variables declared by `S`.
    ///
    /// # Examples
    ///
    /// ```
    /// use confique::Config;
    /// use rust_config_tree::ConfiqueEnvProvider;
    ///
    /// #[derive(Config)]
    /// struct AppConfig {
    ///     #[config(env = "APP_MODE", default = "demo")]
    ///     mode: String,
    /// }
    ///
    /// let provider = ConfiqueEnvProvider::new::<AppConfig>();
    /// # let _ = provider;
    /// ```
    pub fn new<S>() -> Self
    where
        S: Config,
    {
        let mut env_to_path = HashMap::<String, String>::new();
        let mut path_to_env = HashMap::<String, String>::new();

        collect_env_mapping(&S::META, "", &mut env_to_path, &mut path_to_env);

        let env_to_path = Arc::new(env_to_path);
        let path_to_env = Arc::new(path_to_env);
        let map_for_filter = Arc::clone(&env_to_path);

        let env = Env::raw().filter_map(move |env_key| {
            let lookup_key = env_key.as_str().to_ascii_uppercase();

            map_for_filter
                .get(&lookup_key)
                .cloned()
                .map(Uncased::from_owned)
        });

        Self { env, path_to_env }
    }
}

/// Supplies Figment data and source labels for schema-declared environment variables.
impl Provider for ConfiqueEnvProvider {
    /// Builds metadata used by Figment source tracing.
    ///
    /// # Arguments
    ///
    /// - `self`: Environment provider whose path-to-variable mapping should be
    ///   exposed in metadata.
    ///
    /// # Returns
    ///
    /// Returns Figment metadata that renders schema paths as native env names.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
    fn metadata(&self) -> Metadata {
        let path_to_env = Arc::clone(&self.path_to_env);

        Metadata::named("environment variable").interpolater(move |_profile, keys| {
            let path = keys.join(".");

            path_to_env.get(&path).cloned().unwrap_or(path)
        })
    }

    /// Reads configured environment variables into Figment data.
    ///
    /// # Arguments
    ///
    /// - `self`: Environment provider wrapping the filtered Figment env source.
    ///
    /// # Returns
    ///
    /// Returns Figment data grouped by profile, or a Figment error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        self.env.data()
    }
}

/// Recursively maps schema field paths to their declared environment variables.
///
/// # Arguments
///
/// - `meta`: `confique` metadata node to inspect.
/// - `prefix`: Dot-separated field path prefix for `meta`.
/// - `env_to_path`: Output map from uppercase environment names to field paths.
/// - `path_to_env`: Output map from field paths to declared environment names.
///
/// # Returns
///
/// Returns no value; both output maps are updated in place.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn collect_env_mapping(
    meta: &'static Meta,
    prefix: &str,
    env_to_path: &mut HashMap<String, String>,
    path_to_env: &mut HashMap<String, String>,
) {
    for field in meta.fields {
        let path = if prefix.is_empty() {
            field.name.to_owned()
        } else {
            format!("{prefix}.{}", field.name)
        };

        match field.kind {
            FieldKind::Leaf { env: Some(env), .. } => {
                // Keep both directions: Figment needs env -> path for loading,
                // while metadata interpolation needs path -> env for tracing.
                env_to_path.insert(env.to_ascii_uppercase(), path.clone());
                path_to_env.insert(path, env.to_owned());
            }
            FieldKind::Leaf { env: None, .. } => {}
            FieldKind::Nested { meta } => {
                collect_env_mapping(meta, &path, env_to_path, path_to_env);
            }
        }
    }
}
