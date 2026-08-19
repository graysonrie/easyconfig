use std::{
    io::{BufReader, BufWriter},
    marker::PhantomData,
    path::PathBuf,
};

use serde::{Serialize, de::DeserializeOwned};

pub enum SaveTo {
    AppData,
    Custom(PathBuf),
}

/// A holder for a config file
/// Usage:
/// ```rust
/// let holder = ConfigHolder::<MyConfig>::new(SaveTo::AppData, "config");
/// let config = holder.get_or_create().unwrap();
/// ```
pub struct ConfigHolder<T> {
    _type: PhantomData<T>,
    save_to: SaveTo,
    app_name: &'static str,
    config_name: &'static str,
}

impl<T> ConfigHolder<T>
where
    T: Default + Serialize + DeserializeOwned,
{
    pub fn new(save_to: SaveTo, app_name: &'static str, config_name: &'static str) -> Self {
        Self {
            _type: PhantomData,
            save_to,
            app_name,
            config_name,
        }
    }

    /// Gets or creates the config if it doesn't already exist
    pub fn get_or_create(&self) -> anyhow::Result<T> {
        let file_path = self.get_save_path()?;

        if !file_path.exists() {
            let value = T::default();
            let file = std::fs::File::create(&file_path).map_err(|e| anyhow::anyhow!(e))?;
            let writer = BufWriter::new(file);
            serde_json::to_writer(writer, &value).map_err(|e| anyhow::anyhow!(e))?;
        }

        let file = std::fs::File::open(file_path).map_err(|e| anyhow::anyhow!(e))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| anyhow::anyhow!(e))
    }

    /// You should use `get_or_create` first
    pub fn write(&self, value: &T) -> anyhow::Result<()> {
        let file_path = self.get_save_path()?;

        let file = std::fs::File::create(&file_path).map_err(|e| anyhow::anyhow!(e))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &value).map_err(|e| anyhow::anyhow!(e))
    }

    /// Gets the save path, including the '.json' suffix
    pub fn get_save_path(&self) -> anyhow::Result<PathBuf> {
        let save_dir = self
            .get_save_dir()
            .ok_or(anyhow::anyhow!("Save dir not found"))?;

        Ok(save_dir.join(format!("{}.json", self.config_name)))
    }

    fn get_save_dir(&self) -> Option<PathBuf> {
        match self.save_to {
            SaveTo::AppData => dirs::data_dir().map(|path| path.join(self.app_name)),
            SaveTo::Custom(ref path_buf) => Some(path_buf.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::*;

    #[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
    struct CustomConfig {
        pub name: String,
        pub age: u32,
        pub favorite_numbers: Vec<u32>,
    }

    #[test]
    fn get_or_create_returns_default_when_no_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        let holder: ConfigHolder<CustomConfig> = ConfigHolder::new(
            SaveTo::Custom(dir.path().to_path_buf()),
            "test_app",
            "test_config",
        );

        let config = holder.get_or_create().unwrap();

        assert_eq!(config.name, "");
        assert_eq!(config.age, 0);
        assert!(config.favorite_numbers.is_empty());
    }

    #[test]
    fn get_or_create_persists_default_to_disk() {
        let dir = tempfile::tempdir().unwrap();
        let holder: ConfigHolder<CustomConfig> = ConfigHolder::new(
            SaveTo::Custom(dir.path().to_path_buf()),
            "test_app",
            "test_config",
        );

        holder.get_or_create().unwrap();

        let file_path = dir.path().join("test_config.json");
        assert!(file_path.exists());
    }

    #[test]
    fn write_then_read_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let holder: ConfigHolder<CustomConfig> = ConfigHolder::new(
            SaveTo::Custom(dir.path().to_path_buf()),
            "test_app",
            "my_cfg",
        );

        let config = CustomConfig {
            name: "Alice".into(),
            age: 30,
            favorite_numbers: vec![7, 42],
        };

        holder.write(&config).unwrap();
        let loaded = holder.get_or_create().unwrap();

        assert_eq!(loaded, config);
    }

    #[test]
    fn get_or_create_does_not_overwrite_existing() {
        let dir = tempfile::tempdir().unwrap();
        let holder: ConfigHolder<CustomConfig> =
            ConfigHolder::new(SaveTo::Custom(dir.path().to_path_buf()), "test_app", "keep");

        let config = CustomConfig {
            name: "Bob".into(),
            age: 25,
            favorite_numbers: vec![1, 2, 3],
        };
        holder.write(&config).unwrap();

        let loaded = holder.get_or_create().unwrap();
        assert_eq!(loaded, config);
    }

    #[test]
    fn get_save_path_includes_json_extension() {
        let dir = tempfile::tempdir().unwrap();
        let holder: ConfigHolder<CustomConfig> = ConfigHolder::new(
            SaveTo::Custom(dir.path().to_path_buf()),
            "test_app",
            "settings",
        );

        let path = holder.get_save_path().unwrap();
        assert_eq!(path.file_name().unwrap(), "settings.json");
    }
}
