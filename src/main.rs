use ansi_term::Colour;
use directories_next::BaseDirs;
use serde_yaml;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::os::unix::fs;

const DEFAULT_FILE: &str = "manifest.yml";

#[derive(Deserialize, Serialize, Debug)]
enum LocationType {
    Home,
    Config,
}

impl LocationType {
    fn path<'a>(&self, base_dirs: &'a BaseDirs) -> &'a Path {
        match self {
            Self::Home => base_dirs.home_dir(),
            Self::Config => base_dirs.config_dir(),
        }
    }
}

impl Default for LocationType {
    fn default() -> Self {
        LocationType::Config
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct DotFileItem {
    pub source: String,
    pub destination: String,
    #[serde(default)]
    pub location: LocationType,
}

impl DotFileItem {
    pub fn install(&self, base_dirs: &BaseDirs, source_dir: &Path) {
        let mut install_dir = self.location.path(base_dirs).to_path_buf();
        install_dir.push(&self.destination);
        let destination = install_dir.as_path();

        let mut current_dir = source_dir.to_path_buf();
        current_dir.push(&self.source);
        let source = current_dir.as_path();

        if destination.exists() {
            let link_type_res = destination.read_link();
            match link_type_res {
                Ok(link_info) => {
                    if source.canonicalize().unwrap() == link_info.canonicalize().unwrap() {
                        println!("{}: Correct link already exists", Colour::Yellow.paint(&self.source));
                    } else {
                        println!("{}: Incorrect link already exists ({:?})", Colour::Red.paint(&self.source), link_info.canonicalize().unwrap());
                    }
                }
                _ => {
                    println!("{}: Found non-linked file {:?}", Colour::Red.paint(&self.source), &destination);
                }
            }
        } else {
            fs::symlink(&source, &destination).unwrap();
            println!("{}: Created new symlink: {:?}", Colour::Green.paint(&self.source), &destination);
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct ManifestContent {
    pub files: Vec<DotFileItem>,
}

fn main() {
    let config_file = std::env::args().nth(1).unwrap_or(String::from(DEFAULT_FILE));    
    let base_dirs = BaseDirs::new().unwrap();
    let install_dir = Path::new(&config_file).parent().unwrap();

    let contents = std::fs::read_to_string(&config_file).expect("Could not given file");
    let manifest: ManifestContent = serde_yaml::from_str(&contents).unwrap();

    for f in manifest.files {
        f.install(&base_dirs, install_dir);
    }
}

