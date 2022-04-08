use std::collections::HashMap;
use std::io::{Write, Read, BufRead};
use std::path::Path;
use std::{io, fs};

// ========================= //
// ========= UTILS ========= //
// ========================= //

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

// ========================= //
// ========= IMAGE ========= //
// ========================= //

/// Une image au format svg.
pub struct Image {
    /// Le contenu de l'image.
    content: Box<[u8]>,
}

impl FileLoad for Image {
    fn load(mut file: fs::File) -> io::Result<Self> {
        let mut content = Vec::with_capacity(file.metadata()?.len() as usize);
        file.read_to_end(&mut content)?;
        Ok(Self { content: content.into_boxed_slice() })
    }
}

impl Image {
    /// Ecris l'image avec writer.
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.content)
    }
}

// ========================= //
// ========= MODEL ========= //
// ========================= //

/// Une partie d'un modèle.
pub enum ModelPart {
    /// Du texte.
    Text(Box<[u8]>),

    /// Une référence à un argument.
    Argument(usize),
}

/// Un modèle d'image.
pub struct Model {
    /// Les parties qui composent le modèle.
    parts: Vec<ModelPart>,
}

impl FileLoad for Model {
    fn load(file: fs::File) -> io::Result<Self>{
        let mut buffer: Vec<u8> = Vec::with_capacity(file.metadata()?.len() as usize);
        let mut parts: Vec<ModelPart> = Vec::with_capacity(20);

        // On lit le fichier ligne par lignes
        let lines = io::BufReader::new(file).lines();
        for line in lines {
            let line = line?;
            let trim_line = line.trim();

            // On teste si la ligne est une référence à un argument
            if trim_line.starts_with("#GET ") {
                if let Ok(index) = trim_line[5..].parse::<usize>() {
                    // On ajoute le texte du buffer (si il y en a)
                    if buffer.len() > 0 {
                        buffer.push(b'\n');
                        parts.push(ModelPart::Text(buffer.clone().into_boxed_slice()));
                        buffer.clear();
                        buffer.push(b' ');
                    }

                    // On ajoute la référence à un argument
                    parts.push(ModelPart::Argument(index));
                    continue;
                }
            }

            // Sinon on ajoute la ligne au buffer
            if buffer.len() > 0 { buffer.push(b'\n'); }
            buffer.append(&mut line.into_bytes());
        }

        // On ajoute le texte restant (si il y en a)
        if buffer.len() > 0 {
            buffer.push(b'\n');
            parts.push(ModelPart::Text(buffer.clone().into_boxed_slice()));
        }
        
        // On retourne le modèle
        Ok(Model { parts })
    }
}

impl Model {
    /// Retourne les parties du modèle.
    pub fn parts(&self) -> &Vec<ModelPart> {
        &self.parts
    }
}

// ========================= //
// ======== GENERATE ======= //
// ========================= //

/// Génère une image à partir d'un modèle et des arguments fournis et l'écris avec writer.
pub fn generate<W: Write>(writer: &mut W, model: &Model, images: &HashMap<String, Image>, args: &[String]) -> io::Result<()> {
    for part in model.parts() {
        match part {
            ModelPart::Text(text) => writer.write_all(text)?,
            ModelPart::Argument(index) => match args.get(*index) {
                Some(arg) => match images.get(arg) {
                    Some(image) => image.write(writer)?,
                    _ => writer.write_all(arg.as_bytes())?,
                },
                _ => return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Argument non fournis: {}", index)
                )),
            }
        }
    }
    Ok(())
}
