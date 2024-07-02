use std::path::{Path, PathBuf};

const RESOURCES_FOLDER_NAME: &str = "sedavm";

pub fn resources_home_dir(sedad_home: &Path) -> PathBuf {
    sedad_home.join(RESOURCES_FOLDER_NAME)
}
