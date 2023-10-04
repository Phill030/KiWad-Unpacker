use std::{env, path::PathBuf, sync::mpsc::channel};

pub mod wad;
use crate::wad::WadRework;

fn main() {
    // env_logger::Builder::filter_level(&mut Builder::new(), log::LevelFilter::Info).init();
    let args: Vec<String> = env::args().collect();
    let exe_path = &args[0];

    if args.len() >= 2 && args.iter().skip(1).all(|a| a.ends_with(".wad")) {
        println!("Unpacking, This may take a moment... ☕");

        args.iter().skip(1).for_each(|arg| {
            let arg_path = PathBuf::from(arg);

            let mut buffer = std::fs::read(&arg_path).unwrap_or(vec![]);
            let mut wad = WadRework::new(&mut buffer).unwrap();

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

    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl+C handler");

    println!("Press CTRL+C to exit!");
    rx.recv().expect("Could not receive from channel.");
    println!("Exiting...");
}
