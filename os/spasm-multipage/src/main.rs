use clap::{App, Arg};
use spasm_multipage::FileInfo;
use std::ffi::OsStr;
use std::path::PathBuf;

fn main() {
    let matches = App::new("spasm-multipage")
        .arg(
            Arg::with_name("include-dir")
                .short("I")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
        )
        .arg(Arg::with_name("source").multiple(true).required(true))
        .get_matches();

    let include_paths: Vec<&OsStr> = matches
        .values_of_os("include-dir")
        .into_iter()
        .flatten()
        .collect();

    let files: Vec<(PathBuf, FileInfo)> = matches
        .values_of_os("source")
        .into_iter()
        .flatten()
        .map(|p| {
            let path: PathBuf = p.into();
            let f = std::fs::File::open(&path).expect("Unable to open input file");
            let info = FileInfo::parse(std::io::BufReader::new(f));
            (path, info)
        })
        .collect();

    let graph = spasm_multipage::analyze(&files);
    for node in graph.node_indices() {
        let (ref path, ref info) = graph[node];
        eprintln!("{}: page {:#04x}", path.to_string_lossy(), info.page);
        eprintln!("Exports:");
        for export in info.exports.iter() {
            eprintln!("\t{}", export);
        }
        eprintln!("Imports:");
        for import in info.imports.iter() {
            eprintln!("\t{}", import);
        }
    }

    spasm_multipage::build_all(&graph, &include_paths);

    let nicely_labelled = graph.map(
        |_node, value| value.0.to_string_lossy(),
        |_edge, value| value,
    );
    let dot = petgraph::dot::Dot::new(&nicely_labelled);
    println!("{}", dot);
}
