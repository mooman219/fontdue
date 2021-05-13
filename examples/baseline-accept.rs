use std::fs;
use walkdir::WalkDir;

fn copy_changed_baselines() {
    let reference_path = "./resources/baselines/reference/characters";
    let local_path = "./resources/baselines/local/characters";
    let mut delete_count = 0;
    let mut copy_count = 0;
    for entry in WalkDir::new(local_path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_dir() {
            continue;
        }
        let ext = entry.path().extension();
        match ext {
            Some(str) => {
                if str == "delete" {
                    delete_count += 1;
                    let dest_path = std::path::Path::new(reference_path)
                        .join(entry.path().parent().unwrap().file_name().unwrap())
                        .join(format!("{}.png", entry.path().file_stem().unwrap().to_string_lossy()));
                    fs::remove_file(dest_path).ok();
                    continue;
                }
            }
            None => continue,
        }
        copy_count += 1;
        fs::create_dir_all(entry.path().parent().unwrap()).ok();
        let dest_path = std::path::Path::new(reference_path)
            .join(entry.path().parent().unwrap().file_name().unwrap())
            .join(entry.file_name());
        fs::write(dest_path, fs::read(entry.path()).unwrap()).ok();
    }
    if copy_count > 0 {
        println!("{} file(s) copied.", copy_count);
    }
    if delete_count > 0 {
        println!("{} file(s) deleted.", delete_count);
    }
}

fn main() {
    copy_changed_baselines();
}
