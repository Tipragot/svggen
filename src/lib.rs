use std::collections::HashMap;
use std::io::Write;
use std::io;

use image::Image;
use model::*;

mod utils;
mod image;
mod model;

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
