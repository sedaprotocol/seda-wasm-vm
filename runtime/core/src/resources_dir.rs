use std::path::PathBuf;

const RESOURCES_FOLDER_NAME: &str = "sedavm";

/// Stores the resources into a common directory
/// When XDG_DATA_HOME is set all resources will be stored in $XDG_DATA_HOME/seda
/// Otherwise in $HOME/.seda
pub fn resources_home_dir() -> PathBuf {
    let xdg_home = std::env::var("XDG_DATA_HOME");

    match xdg_home {
        Ok(dir) => {
            let mut home = PathBuf::from(dir);

            home.push(RESOURCES_FOLDER_NAME);
            home
        }
        Err(_) => {
            let mut home = home::home_dir()
                .expect("HOME directory could not be determined, which is required for this app to function");

            home.push(format!(".{RESOURCES_FOLDER_NAME}"));
            home
        }
    }
}
