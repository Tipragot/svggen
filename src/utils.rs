use std::collections::HashMap;
use std::path::Path;
use std::{io, fs};

/// Permet de créer un objet à partir d'un fichier.
pub trait FileLoad: Sized {
    /// Crée un objet à partir d'un fichier.
    fn load(file: fs::File) -> io::Result<Self>;

    /// Récupère toutes les objets d'un dossier.
    fn load_folder<P: AsRef<Path>>(folder: P) -> HashMap<String, Self> {
        let mut objects = HashMap::new();
        if let Ok(directory) = fs::read_dir(folder) {
            for entry in directory {
                let entry = match entry {
                    Ok(entry) => entry,
                    _ => continue,
                };

                let name = match entry.file_name().to_str() {
                    Some(name) => match name.rfind('.') {
                        Some(index) => name[..index].to_owned(),
                        _ => continue,
                    },
                    _ => continue,
                };

                let file = match fs::File::open(entry.path()) {
                    Ok(file) => file,
                    _ => continue,
                };

                match Self::load(file) {
                    Ok(obj) => objects.insert(name, obj),
                    _ => continue,
                };
            }
        }
        objects
    }
}