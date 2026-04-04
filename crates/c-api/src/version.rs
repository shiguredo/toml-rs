/// TOML バージョン。
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum TomlVersion {
    /// TOML v1.0.0
    TOML_VERSION_1_0 = 0,
    /// TOML v1.1.0
    TOML_VERSION_1_1 = 1,
}

impl From<TomlVersion> for shiguredo_toml::TomlVersion {
    fn from(v: TomlVersion) -> Self {
        match v {
            TomlVersion::TOML_VERSION_1_0 => shiguredo_toml::TomlVersion::V1_0,
            TomlVersion::TOML_VERSION_1_1 => shiguredo_toml::TomlVersion::V1_1,
        }
    }
}
