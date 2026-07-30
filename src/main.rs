use crate::library::Library;
use clap::Parser;
use miette::bail;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub mod errors;
mod library;
pub mod reader;

#[derive(Parser)]
#[command(name = "kiwad_unpacker", about = "Extract files from KIWAD archives")]
struct Cli {
    #[arg(short, long, default_value = "./output")]
    output: PathBuf,

    #[arg(required = true, num_args = 1..)]
    inputs: Vec<String>,
}

struct WadGroup {
    wad: PathBuf,
    filters: Option<Vec<String>>,
}

fn group_inputs(inputs: &[String]) -> miette::Result<Vec<WadGroup>> {
    let mut groups = Vec::new();
    let mut current_wad: Option<PathBuf> = None;
    let mut current_filters = Vec::new();

    for arg in inputs {
        let path = Path::new(arg);
        if path.extension().map(|e| e.eq_ignore_ascii_case("wad")).unwrap_or(false) {
            if let Some(wad) = current_wad.take() {
                groups.push(WadGroup {
                    wad,
                    filters: if current_filters.is_empty() {
                        None
                    } else {
                        Some(std::mem::take(&mut current_filters))
                    },
                });
            }
            current_wad = Some(path.to_owned());
        } else {
            if current_wad.is_none() {
                bail!("Unexpected argument '{arg}' before any .wad file");
            }
            current_filters.push(arg.clone());
        }
    }

    if let Some(wad) = current_wad {
        groups.push(WadGroup {
            wad,
            filters: if current_filters.is_empty() { None } else { Some(current_filters) },
        });
    } else {
        bail!("No .wad file given");
    }

    Ok(groups)
}

fn main() -> miette::Result<()> {
    miette::set_panic_hook();

    let cli = Cli::parse();
    let groups = group_inputs(&cli.inputs)?;

    for group in groups {
        println!("Processing {}", group.wad.display());

        let mut lib = Library::new(&group.wad)?;
        let mut save_path = cli.output.join(group.wad.file_name().unwrap());

        println!("Extracting files...");
        let filter_set = group.filters.map(|f| f.into_iter().collect::<HashSet<String>>());

        lib.extract(&mut save_path, filter_set.as_ref())
            .map_err(|e| miette::miette!("Failed to extract files: {}", e))?;
    }

    Ok(())
}
