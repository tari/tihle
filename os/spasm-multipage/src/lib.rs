#[macro_use]
extern crate log;

use petgraph::Graph;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub enum Error {
    CircularImports,
    AssemblerNotRun(std::io::Error),
    AssemblerError(std::process::ExitStatus),
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    /// The page this file is declared as.
    ///
    /// Example declaration:
    /// ```text
    /// ; MULTIPAGE:PAGE:1B
    /// ```
    pub page: u8,
    /// The symbols exported by this file.
    ///
    /// Example declaration:
    /// ```text
    /// ; MULTIPAGE:EXPORT:FooBar
    /// ```
    ///
    /// When building a file that imports this symbol, `FOOBAR` will be defined to be the
    /// address of the symbol on this page, and `FOOBAR_PAGE` will be defined as the page
    /// of this file.
    ///
    /// Note that symbols are converted to upper case, because spasm (by default) is
    /// case-insensitive.
    pub exports: HashSet<String>,
    /// The symbols imported by this file.
    ///
    /// Example declaration:
    /// ```text
    /// ; MULTIPAGE:IMPORT:FooBar
    /// ```
    ///
    /// Note that symbols are converted to upper case, because spasm (by default) is
    /// case-insensitive.
    pub imports: HashSet<String>,
}

impl FileInfo {
    /// Parse some input, emitting information from the contained directives.
    pub fn parse<R: BufRead>(r: R) -> FileInfo {
        let page_pat = Regex::new(r"^; MULTIPAGE:PAGE:([[:xdigit:]]{2})$").unwrap();
        let import_pat = Regex::new(r"; MULTIPAGE:IMPORT:(\S+)$").unwrap();
        let export_pat = Regex::new(r"; MULTIPAGE:EXPORT:(\S+)$").unwrap();

        let mut page: Option<u8> = None;
        let mut imports: HashSet<String> = HashSet::new();
        let mut exports: HashSet<String> = HashSet::new();

        for (line_no, line) in r.lines().enumerate() {
            let line = line.unwrap();
            if let Some(captures) = page_pat.captures(&line) {
                let was = page
                    .replace(u8::from_str_radix(captures.get(1).unwrap().as_str(), 16).unwrap());
                if was.is_some() {
                    panic!("Duplicate PAGE declaration found at line {}", line_no);
                }
            } else if let Some(captures) = import_pat.captures(&line) {
                let import = captures.get(1).unwrap().as_str().to_uppercase();
                imports.insert(import);
            } else if let Some(captures) = export_pat.captures(&line) {
                let export = captures.get(1).unwrap().as_str().to_uppercase();
                if exports.contains(&export) {
                    panic!(
                        "Export {} is declared multiple times (second at line {})",
                        export, line_no
                    );
                }
                exports.insert(export);
            }
        }

        if let Some(page) = page {
            FileInfo {
                page,
                imports,
                exports,
            }
        } else {
            panic!("No PAGE declaration found");
        }
    }
}

/// Generate a dependency graph between all of the provided files.
pub fn analyze(sources: &[(PathBuf, FileInfo)]) -> DependencyGraph {
    let mut g: DependencyGraph = Graph::new();
    let mut exporters: HashMap<&str, petgraph::graph::NodeIndex> = HashMap::new();

    for (path, info) in sources.iter() {
        let node = g.add_node((path.clone(), info.clone()));
        for symbol in info.exports.iter() {
            exporters.insert(symbol, node);
        }
    }

    for (importer, source) in g.node_indices().zip(sources.iter()) {
        for import in source.1.imports.iter() {
            let exporter = match exporters.get(import.as_str()) {
                Some(n) => n,
                None => {
                    panic!("No source exports symbol {}", import);
                }
            };

            g.add_edge(*exporter, importer, import.to_string());
        }
    }

    g
}

/// File dependency graph; files linked by their imports.
///
/// This is a directed graph, where edges go from the file that
/// exports a symbol to all that import it.
pub type DependencyGraph = Graph<(PathBuf, FileInfo), String>;

pub fn build_all<P: AsRef<Path>>(
    depgraph: &DependencyGraph,
    include_paths: &[P],
) -> Result<(), Error> {
    assert!(depgraph.is_directed());
    let mut global_symbols: HashMap<String, (u8, u16)> = HashMap::new();

    let nodes = match petgraph::algo::toposort(&depgraph, None) {
        Ok(f) => f,
        Err(_) => {
            return Err(Error::CircularImports);
        }
    };

    // Canonicalize include paths as required for build() because it changes the
    // working directory for spasm.
    let include_paths: Vec<PathBuf> = include_paths
        .iter()
        .map(|p| {
            p.as_ref()
                .canonicalize()
                .expect("Failed to canonicalize include path")
        })
        .collect();

    for node in nodes {
        let &(ref path, ref info) = &depgraph[node];
        let mut outfile: PathBuf = path.clone();
        outfile.set_extension("bin");

        info!("Build {:?} (page {:#04x})", path, info.page);
        let mut exported_values = build(
            path,
            &outfile,
            &include_paths,
            &info.exports,
            &global_symbols,
        )?;
        let with_pages = exported_values
            .drain()
            .map(|(symbol, addr)| (symbol, (info.page, addr)));
        global_symbols.extend(with_pages);
    }

    Ok(())
}

/// Assemble a single source file
///
/// `file` is the path of the file to assemble, and `target` is the path
/// to write output to. The extension of the target file tells spasm
/// what format to output.
///
/// `include_dirs` specifies additional directories to search for files
/// included via #include in the source; they are passed to spasm via
/// the -I option.
///
/// `exports` is the list of symbols this file exports, which will be
/// extracted from the output and returned, mapping each symbol to its
/// address.
///
/// `imports` should map the symbol to the page and address for each
/// declared import in the file.
fn build<P: AsRef<Path>>(
    file: &Path,
    target: &Path,
    include_dirs: &[P],
    exports: &HashSet<String>,
    imports: &HashMap<String, (u8, u16)>,
) -> Result<HashMap<String, u16>, Error> {
    eprintln!("Building {:?}", file);
    let build_dir = tempdir::TempDir::new("spasm-multipage-build").unwrap();
    // Copy source file to temporary directory because the label file always get written next to it
    let tmp_source = {
        let mut p = build_dir.path().to_owned();
        p.push("src.asm");
        p
    };
    std::fs::copy(file, &tmp_source).expect("Unable to copy input file to temporary directory");

    let mut spasm = Command::new("spasm");
    spasm.current_dir(build_dir.path());
    // Output a listing file
    spasm.arg("-T");

    // Generate a symbol table, to extract exports from
    spasm.arg("-L");
    for include_dir in include_dirs {
        spasm.arg("-I");
        spasm.arg(include_dir.as_ref().as_os_str());
    }

    // Define all the imports
    for (label, (ref page, ref addr)) in imports.iter() {
        spasm.arg(format!("-D{}_PAGE=${:x}", label, page));
        spasm.arg(format!("-D{}=${:x}", label, addr));
    }

    // Read the input file (temporary copy)
    spasm.arg(&tmp_source);

    // The symbols file gets written next to the output, so must output to the build dir
    // as well.
    let output_filename = {
        let mut p = build_dir.path().to_owned();
        p.push("out.bin");
        if let Some(ext) = target.extension() {
            p.set_extension(ext);
        }
        p
    };
    spasm.arg(&output_filename);

    match spasm.status() {
        Ok(s) if s.success() => {}
        Ok(s) => {
            return Err(Error::AssemblerError(s));
        }
        Err(e) => {
            return Err(Error::AssemblerNotRun(e));
        }
    }

    // Copy output to the provided output path
    std::fs::copy(&output_filename, target).expect("Failed to copy output from tempdir");

    // Copy the listing file too
    {
        let mut src = output_filename.clone();
        src.set_extension("lst");
        let mut dst = target.to_owned();
        dst.set_extension("lst");
        std::fs::copy(src, dst).expect("Failed to copy listing file from tempdir");
    }

    // Read symbol table and extract exports
    let mut export_values = HashMap::new();
    let symbols_file = {
        let mut p = build_dir.path().to_owned();
        p.push("out.lab");
        p
    };
    assert!(
        symbols_file.exists(),
        "Symbols should be output to {:?}",
        symbols_file
    );
    let symbols_file =
        File::open(symbols_file).expect("Unable to open symbol table to fetch exports");
    for line in BufReader::new(symbols_file).lines() {
        let line = line.unwrap();
        trace!("SYMFILE: {}", line);
        // LABEL = $0000
        let mut parts = line.split(" = $");
        let label = parts.next().expect("Unexpected format for symbol table");

        if exports.contains(label) {
            let value = parts.next().expect("Unexpected format for symbol table");
            let value = u16::from_str_radix(value, 16).expect("Unusable value for symbol");
            export_values.insert(label.to_string(), value);
        }
    }

    for expected in exports.iter() {
        assert!(
            export_values.contains_key(expected),
            "{:?} declared export {} but the symbol was not exported",
            file,
            expected
        );
    }
    debug!("Exports: {:?}", export_values);

    Ok(export_values)
}

pub fn autobuild<P: AsRef<Path>>(files: &[P], include_dirs: &[P]) -> Result<(), Error> {
    let parsed_files = files
        .iter()
        .map(|path| {
            let f = std::fs::File::open(path).expect("Unable to open source file for parsing");
            let info = FileInfo::parse(BufReader::new(f));
            (path.as_ref().to_owned(), info)
        })
        .collect::<Vec<_>>();
    let graph = analyze(&parsed_files);

    build_all(&graph, &include_dirs)
}
