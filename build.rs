use std::{env, fs::File, io::{BufWriter, Write}, path::Path};
use heck::ToUpperCamelCase;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("commands.rs");
    let mut file = BufWriter::new(File::create(dest_path).unwrap());

    writeln!(file, "use std::collections::HashMap;")?;
    writeln!(file, "use once_cell::sync::Lazy;")?;
    
    let mut commands = vec![];

    for entry in WalkDir::new("src/cmd").into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {continue;}

        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if stem != "mod" {
                commands.push(stem.to_string());
            }
        }
    }

    // Generate the HashMap definition and insertions.
    writeln!(file, "static COMMANDS: Lazy<HashMap<String, Box<dyn Command>>> = Lazy::new(|| {{")?;
    writeln!(file, "    let mut m: HashMap<String, Box<dyn Command>> = HashMap::new();")?;

    for cmd in &commands {
        // e.g., for "say.rs", creates `SayCommand`
        let struct_name = cmd.to_upper_camel_case();
        writeln!(
            file,
            "    m.insert(\"{}\".to_string(), Box::new({}::{}Command));",
            cmd, cmd, struct_name
        )?;
    }
    writeln!(file, "m\n}});")?;

    Ok(())
}
