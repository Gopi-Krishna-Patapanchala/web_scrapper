use std::fs;

fn delete_folders(folder_paths: Vec<&str>) {
    for folder_path in folder_paths {
        match fs::remove_dir_all(folder_path) {
            Ok(_) => println!("Folder '{}' deleted successfully.", folder_path),
            Err(e) => eprintln!("Error deleting folder '{}': {}", folder_path, e),
        }
    }
}

pub fn clean_folders() {
    let folders_to_delete = vec![
        "./Match_details",
        "./staging_area"
      
    ];

    delete_folders(folders_to_delete);
}
