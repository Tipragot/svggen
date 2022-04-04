use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::io;

use utils::FileLoad;
use image::Image;
use model::*;

mod utils;
mod image;
mod model;

/// Une commande de génération d'image.
pub struct Command {
    /// Le nom du modèle à utiliser.
    pub model_name: String,

    /// Les arguments de la commande.
    pub args: Vec<String>,
}

/// Un générateur d'images.
pub struct Generator {
    /// Les images chargé dans le générateur.
    images: HashMap<String, Image>,

    /// Les modèles chargé dans le générateur.
    models: HashMap<String, Model>,
}

impl Generator {
    /// Crée un générateur qui utilise les images et les modèles des dossiers fournis.
    pub fn new<P: AsRef<Path>>(images_folder: P, models_folder: P) -> Self {
        Self {
            images: Image::load_folder(images_folder),
            models: Model::load_folder(models_folder),
        }
    }

    /// Génaire une image à partir d'une commande et l'écris avec writer.
    pub fn generate<W: Write>(&self, writer: &mut W, command: &Command) -> io::Result<()> {
        // Récupération du modèle utilisé
        let model = match self.models.get(&command.model_name) {
            Some(model) => model,
            _ => return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Modelèle introuvable: {}", command.model_name)
            ))
        };

        // On écris les parties du modèle
        for part in model.parts() {
            match part {
                ModelPart::Text(text) => writer.write_all(text)?,
                ModelPart::Argument(index) => match command.args.get(*index) {
                    Some(arg) => match self.images.get(arg) {
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

        // L'opération a réussit
        Ok(())
    }
}
