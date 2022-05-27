use std::io;
use std::path::Path;

pub fn compile_schemas(input_path: &Path) -> io::Result<()> {

    // read schema files
    let input_files : Vec<_> = input_path.read_dir()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().to_str().unwrap().ends_with(".schema.yaml"))
        .collect();

    let output_path = Path::new(env!("OUT_DIR"));

    // generate rust code files
    for entry in input_files {
        if let Some(input_file_name) =  entry.path().file_name() {
            let prefix_name : String = input_file_name.to_string_lossy().split('.').take(1).collect();
            let output_file_name = output_path.join(format!("{}.rs", &prefix_name));

            schemafy_lib::Generator::builder()
                .with_root_name_str(&prefix_name)
                .with_input_file(entry.path().as_path())
                .build()
                .generate_to_file(&output_file_name.as_path())
                .unwrap();
        }
    }

    Ok(())
}