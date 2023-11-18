#[cfg(feature = "alias")]
use serde::Deserialize;

#[cfg_attr(feature = "alias", derive(Deserialize))]
pub(crate) struct Aliases {
    pub r#ref: Option<String>,
    pub ref_mut: Option<String>,
    pub keyed_ref: Option<String>,
    pub keyed_ref_mut: Option<String>,
}

impl Aliases {
    pub(crate) fn legacy() -> Self {
        Self {
            r#ref: Some("ServiceRef".into()),
            ref_mut: None,
            keyed_ref: None,
            keyed_ref_mut: None,
        }
    }
}

#[cfg(feature = "alias")]
#[derive(Deserialize)]
pub(crate) struct Dependency {
    pub aliases: Option<Aliases>,
}

#[cfg(feature = "alias")]
#[derive(Deserialize)]
pub(crate) struct Dependencies {
    #[cfg_attr(feature = "alias", serde(alias = "more-di"))]
    pub di: Option<Dependency>,

    #[cfg_attr(feature = "alias", serde(alias = "more-di-macros"))]
    pub di_macros: Option<Dependency>,
}

#[cfg(feature = "alias")]
#[derive(Deserialize)]
pub(crate) struct Manifest {
    pub dependencies: Option<Dependencies>,
}

#[cfg(not(feature = "alias"))]
pub(crate) fn try_get_aliases() -> Option<Aliases> {
    None
}

#[cfg(feature = "alias")]
pub(crate) fn try_get_aliases() -> Option<Aliases> {
    use std::env::var;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    let path = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap_or_default()).join("Cargo.toml");

    if !path.exists() {
        return None;
    }

    let mut input = String::new();

    if let Ok(_) = File::open(path).and_then(|mut f| f.read_to_string(&mut input)) {
        if let Ok(manifest) = toml::from_str::<Manifest>(&input) {
            if let Some(deps) = manifest.dependencies {
                return deps.di.or(deps.di_macros).map_or(None, |d| d.aliases);
            }
        }
    }

    return None;
}
