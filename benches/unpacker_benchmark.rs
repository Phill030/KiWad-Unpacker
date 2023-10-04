// use criterion::{criterion_group, criterion_main, Criterion};
// use kiwad_unpacker::Wad::WadRework;
// use std::path::PathBuf;

// pub fn criterion_benchmark(c: &mut Criterion) {
//     c.bench_function("kiwad-unpacker", |b| {
//         b.iter(|| {
//             let path = PathBuf::from("D:/Wizard101US/Data/GameData/WizardCity-WC_Golem_Tower.wad");

//             let mut save_path = PathBuf::from(&path)
//                 .parent()
//                 .unwrap()
//                 .join("output")
//                 .join(&path.file_name().unwrap());

//             let mut buffer = std::fs::read(&path).unwrap_or(vec![]);
//             let mut wad = WadRework::new(&mut buffer).unwrap();

//             wad.open_all_files(&mut save_path);
//         })
//     });
// }

// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);
