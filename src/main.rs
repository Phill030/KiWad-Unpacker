use crate::library::Library;
use std::{env, path::PathBuf};

mod library;

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_path = &args[0];

    if args.len() >= 2 && args.iter().skip(1).all(|a| a.ends_with(".wad")) {
        println!("Unpacking, This may take a moment... ☕");

        args.iter().skip(1).for_each(|arg| {
            let arg_path = PathBuf::from(arg);

            let mut buffer = std::fs::read(&arg_path).unwrap_or(vec![]);
            let mut wad = Library::new(&mut buffer).unwrap();

            let mut save_path = PathBuf::from(&exe_path)
                .parent()
                .unwrap()
                .join("output")
                .join(arg_path.file_name().unwrap());

            wad.open_all_files(&mut save_path);
        });

        println!("Done!");
    } else {
        eprintln!("No or incorrect file(s) provided. All imported files must have the .wad file extension!");
        eprintln!("Usage: Drag one or multiple .wad files onto this executable.");
    }
}
