use crate::utils::*;
use std::io::BufRead;
use std::{io, fs};

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
    pub parts: Vec<ModelPart>,
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