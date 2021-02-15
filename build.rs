use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn copy<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> std::io::Result<()> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        println!("process: {:?}", &working_path);

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            println!(" mkdir: {:?}", dest);
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed to copy: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    let os_conf_path = env::var_os("HOME").map(|s| Path::new(&s).join(".config"));
    #[cfg(target_os = "windows")]
    let os_conf_path = env::var_os("APPDATA").map(|s| Path::new(s));
    #[cfg(target_os = "macos")]
    let os_conf_path = env::var_os("HOME").map(|s| Path::new(&s).join(".config"));

    let install_path = os_conf_path.unwrap_or(env::current_dir().expect("Couldn't find a place to install to.")).join("ttyper");
    fs::create_dir_all(&install_path)?;

    let resources_path = env::current_dir().expect("Couldn't find the source directory.").join("resources");
    copy(&resources_path, &install_path)?;

    Ok(())
}
