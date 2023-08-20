use ansi_term::Colour;
use anyhow::{bail, Context, Result};
use directories_next::BaseDirs;
use serde::{Deserialize, Serialize};
use std::os::unix::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

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
    pub fn install(&self, base_dirs: &BaseDirs, source_dir: &Path) -> Result<()> {
        let mut install_dir = self.location.path(base_dirs).to_path_buf();
        install_dir.push(&self.destination);
        let destination = install_dir.as_path();

        let mut source_path = source_dir.to_path_buf();
        source_path.push(&self.source);
        let source = source_path.as_path();

        if !source.exists() {
            bail!(format!(
                "Source file '{}' not found",
                source.to_str().unwrap()
            ));
        }

        if destination.exists() {
            let link_type_res = destination.read_link();
            match link_type_res {
                Ok(link_info) => {
                    if source.canonicalize()? == link_info.canonicalize()? {
                        println!(
                            "{}: Correct link already exists",
                            Colour::Yellow.paint(&self.source)
                        );
                    } else {
                        println!(
                            "{}: Incorrect link already exists ({:?})",
                            Colour::Red.paint(&self.source),
                            link_info.canonicalize()?
                        );
                    }
                }
                _ => {
                    println!(
                        "{}: Found non-linked file {:?}",
                        Colour::Red.paint(&self.source),
                        &destination
                    );
                }
            };
        } else {
            let destination_dir = destination.parent().unwrap();
            if !destination_dir.exists() {
                std::fs::create_dir_all(destination_dir)?;
            }
            fs::symlink(&source.canonicalize().unwrap(), &destination)
                .context(format!("{:?} -> {:?}", &source, &destination))?;
            println!(
                "{}: Created new symlink: {:?}",
                Colour::Green.paint(&self.source),
                &destination
            );
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct ManifestContent {
    pub files: Vec<DotFileItem>,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "dotfiles")]
/// A tool for managing dotfiles.
///
/// Dot files are managed using a manifest YAML file. The YAML files should
/// contain a list of files to install. Each containing a `source` and `destination`.
///
/// The `source` should be the path of a file to install relative to the manifest files.
///
/// The `destination` should be the path to install the `source`. By default, this location
/// will be relative to the "config" directory for your system (see XDG Base Directories).
///
/// An additional YAML property `location` can be set to "Home" to make the `destination`
/// relative to the home directory.
struct Opt {
    /// Manifest file containing files to configure.
    #[structopt(parse(from_os_str), default_value = DEFAULT_FILE, short, long)]
    manifest_file: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let base_dirs = BaseDirs::new().unwrap();
    let install_dir = Path::new(&opt.manifest_file).parent().unwrap();

    let contents = std::fs::read_to_string(&opt.manifest_file).expect("Could not given file");
    let manifest: ManifestContent =
        serde_yaml::from_str(&contents).expect("Could not parse manifest contents");

    for f in manifest.files {
        if let Err(err) = f.install(&base_dirs, install_dir) {
            eprintln!("{}: {:?}", Colour::Red.paint("ERROR"), err);
        }
    }
}
