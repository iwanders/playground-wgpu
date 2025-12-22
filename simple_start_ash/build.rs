fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    let src_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut slang_files = Vec::new();

    fn find_slang_files(dir: &str, files: &mut Vec<String>) {
        let entries = std::fs::read_dir(dir).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name().unwrap().to_str().unwrap();
                if filename.ends_with(".slang") {
                    files.push(path.to_string_lossy().to_string());
                }
            } else if path.is_dir() {
                find_slang_files(path.to_str().unwrap(), files);
            }
        }
    }

    find_slang_files(&src_dir, &mut slang_files);

    for file in slang_files.iter() {
        println!("cargo::rerun-if-changed={}", file);
        let file_path = std::path::Path::new(&file);
        // let file_dirname = file_path.parent().unwrap();
        let path_as_spv = file_path.with_extension("spv");
        // let spv_file_basename = path_as_spv.file_stem().unwrap().to_string_lossy();
        let output = std::process::Command::new("slangc")
            .arg(&file_path)
            .arg("-o")
            .arg(&path_as_spv)
            .output()
            .expect("failed to execute slangc");

        if !output.status.success() {
            eprintln!("slangc failed for file: {}", file);
            eprintln!("Error output: {}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(1);
        }
    }
    eprintln!("found files {slang_files:?}");
}
