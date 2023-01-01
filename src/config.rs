#![allow(dead_code)]

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub static ENV_FILE: Lazy<String> = Lazy::new(|| match std::env::var("ENV_FILE") {
    Ok(value) => value,
    Err(_) => ".env".to_string(),
});

/// $action:
/// - default
/// - option
/// - none
///
/// $editable: enable to set value to this field
/// - true:
/// - false
///
macro_rules! generate_config {
    ($(
        $(#[doc = $doc:literal])+
        $name:ident : $ty:ty, $editable:literal, $action:ident $(, $default:expr)?;
    )+) => {
        use crate::util::get_env;
        // use crate::util::get_env_bool;

        #[derive(Serialize, Deserialize, Debug)]
        pub struct ConfigItems {
            $(
                $name: generate_config!(@type $ty, $action),
            )+
        }

        impl ConfigItems {
            $(
                pub fn $name(&self) -> generate_config!(@type $ty, $action) {
                    self.$name.clone()
                }
            )+
        }

        use core::fmt::Display;
        use log::error;

        impl Display for ConfigItems {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                $(
                    write!(f, "\n\x1b[32m[*]\x1b[0m {} => {:?}", stringify!($name), self.$name).unwrap();
                )+
                Ok(())
            }
        }

        #[derive(Serialize, Deserialize, Debug, Default)]
        pub struct BuilderItems {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $name: Option<$ty>,
            )+
        }

        #[derive(Serialize, Deserialize, Debug, Default, Clone)]
        pub struct BuilderItemInfo {
            /// Config item node name
            pub cfg_name: String,
            /// Where config item defined
            pub come_from: String,
        }

        impl BuilderItemInfo {
            pub fn new (cfg_name: &str, come_from: &str) -> Self {
                BuilderItemInfo {
                    cfg_name: cfg_name.to_string(),
                    come_from: come_from.to_string(),
                }
            }
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct ConfigBuilder {
            pub item_count: usize,
            pub builder_items: BuilderItems,
            pub builder_item_info_map: HashMap<String, BuilderItemInfo>
        }

        impl ConfigBuilder {

            /// Init with nothing
            pub fn new() -> Self {
                let mut info = HashMap::new();
                let mut count: usize = 0;
                $(
                    count += 1;
                    info.insert(stringify!($name).to_string(), BuilderItemInfo::new(stringify!($name), "none"));
                )+
                ConfigBuilder {
                    item_count: count,
                    builder_item_info_map: info,
                    builder_items: BuilderItems {
                        $(
                            $name: None,
                        )+
                    }
                }
            }

            // Set name
            $(
                pub fn $name(&mut self, val: Option<$ty>) -> &mut Self {
                    if let Some(val) = val {
                        if $editable {
                            self.builder_items.$name = Some(val);
                            self.builder_item_info_map.insert(stringify!($name).to_string(),
                            BuilderItemInfo::new(stringify!($name), "set"));
                        }
                    }
                    self
                }
            )+

            pub fn from_env() -> anyhow::Result<Self> {
                let mut cfg: ConfigBuilder = ConfigBuilder::new();
                $(
                    if stringify!($action) != "none" {
                        if let Some(value) = generate_config!(@getenv &stringify!($name).to_uppercase(), $action, $ty) {
                            cfg.builder_items.$name = Some(value);
                            cfg.builder_item_info_map.insert(stringify!($name).to_string(),
                            BuilderItemInfo::new(stringify!($name), "env"));
                        }
                    }
                )+
                Ok(cfg)
            }

            pub fn from_file(path: &str) -> anyhow::Result<Self> {
                let mut cfg: ConfigBuilder = ConfigBuilder::new();
                use std::path::PathBuf;
                use crate::util::read_file_string;
                if !PathBuf::from(path).exists() {
                    return Err(anyhow::Error::msg("File not exists"));
                }
                let config_str = read_file_string(path).expect("Read file failed.");
                let items: BuilderItems = serde_json::from_str(&config_str)?;
                $(
                    if let Some(value) = items.$name {
                        cfg.builder_items.$name = Some(value);
                        cfg.builder_item_info_map.insert(stringify!($name).to_string(),
                        BuilderItemInfo::new(stringify!($name), &format!("file://{}", path)));
                    }
                )+
                Ok(cfg)
            }

            pub fn _to_file(&self) {
                todo!();
            }

            pub fn merge(&mut self, cfg: Self) {
                $(
                    if let Some(val) = cfg.builder_items.$name {
                        self.builder_items.$name = Some(val);
                        self.builder_item_info_map.insert(stringify!($name).to_string(),
                            cfg.builder_item_info_map.get(stringify!($name)).unwrap().clone());
                    }
                )+
            }

            pub fn add_env(mut self) -> Self {
                let cfg = ConfigBuilder::from_env().unwrap();
                self.merge(cfg);
                self
            }

            pub fn add_file(mut self, path: &str) -> Self {
                match ConfigBuilder::from_file(path) {
                    Ok(cfg) => self.merge(cfg),
                    Err(e) => error!("{e}")
                };
                self
            }

            pub fn build(&self) -> ConfigItems {
                ConfigItems {
                    $(
                        $name: generate_config!(@build self.builder_items.$name.clone(), $action $(, $default)?),
                    )+
                }
            }
        }

        /// Load default config vars
        impl Default for ConfigBuilder {
            fn default() -> Self {
                ConfigBuilder::new()
            }
        }
    };

    (@type $ty:ty, option) => { Option<$ty> };
    (@type $ty:ty, $id:ident) => { $ty };

    (@build $value:expr, none) => { $value.unwrap() };
    (@build $value:expr, $action: ident) => { $value };
    (@build $value:expr, $action: ident, $default:expr) => { $value.unwrap_or($default) };

    (@init $ty:ty) => { None };
    (@init $ty:ty, $default:expr) => { Some($default) };

    (@getenv $name:expr, none, $ty:ty) => { None };
    (@getenv $name:expr, $action: ident, bool) => { get_env_bool($name) };
    (@getenv $name:expr, $action: ident, $ty:ty) => { get_env($name) };
}

generate_config! {
    /// All ids to download.
    id: Vec<String>, true, none;
    /// Allow downloading flac.
    flac_allowed: bool, true, default, false;
    /// Allow downloading dolby.
    dolby_allowed: bool, true, default, false;
    /// Allow adding picture to audio.
    pic_allowed: bool, true, default, false;
    /// Path to save audio files.
    path: String, true, default, "./".to_string();
    /// File name.
    filename: String, true, default, "".to_string();
    /// Session.
    session: String, true, default, "".to_string();
}
