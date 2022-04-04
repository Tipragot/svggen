use crate::utils::*;
use std::io::Read;
use std::{io, fs};

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