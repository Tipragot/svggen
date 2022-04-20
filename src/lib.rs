use std::collections::HashMap;
use std::io::{Write, BufRead};
use std::path::Path;
use std::{io, fs};

// ========================= //
// ========= UTILS ========= //
// ========================= //

/// An object that can be created from a file.
pub trait FileLoad: Sized {
    /// Creates an object from a file.
    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self>;

    /// Get all objects from a directory (one object per file).
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

                match Self::load(entry.path()) {
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

/// An image (in svg format).
pub struct Image {
    /// The content of the image.
    content: Box<[u8]>,
}

impl FileLoad for Image {
    /// Creates an image from a file.
    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Ok(Self { content: fs::read(path)?.into_boxed_slice() })
    }
}

impl Image {
    /// Create an image with the given content.
    pub fn new(content: Box<[u8]>) -> Self {
        Self { content }
    }

    /// Returns the content of the image.
    pub fn content(&self) -> &[u8] {
        &self.content
    }
}

// ========================= //
// ========= MODEL ========= //
// ========================= //

/// A part of a model.
pub enum ModelPart {
    /// Some text.
    Text(Box<[u8]>),

    /// A reference to an argument (an image or some text).
    Argument(usize),
}

/// An image model.
pub struct Model {
    /// The parts of the model.
    parts: Vec<ModelPart>,
}

impl FileLoad for Model {
    /// Creates a model from a file.
    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self>{
        let file = fs::File::open(path)?;
        let mut buffer: Vec<u8> = Vec::with_capacity(file.metadata()?.len() as usize);
        let mut parts: Vec<ModelPart> = Vec::with_capacity(20);

        // For each line
        let lines = io::BufReader::new(file).lines();
        for line in lines {
            let line = line?;
            let trim_line = line.trim();

            // If the line is an argument reference
            if trim_line.starts_with("#GET ") {
                if let Ok(index) = trim_line[5..].parse::<usize>() {
                    // Add the text buffer to the parts (if it's not empty)
                    if buffer.len() > 0 {
                        buffer.push(b'\n');
                        parts.push(ModelPart::Text(buffer.clone().into_boxed_slice()));
                        buffer.clear();
                        buffer.push(b' ');
                    }

                    // Add the argument reference to the parts
                    parts.push(ModelPart::Argument(index));
                    continue;
                }
            }

            // Else add the line to the text buffer
            if buffer.len() > 0 { buffer.push(b'\n'); }
            buffer.append(&mut line.into_bytes());
        }

        // Add the text buffer to the parts (if it's not empty)
        if buffer.len() > 0 {
            parts.push(ModelPart::Text(buffer.clone().into_boxed_slice()));
        }
        
        // Return the model
        Ok(Model { parts })
    }
}

impl Model {
    /// Create a model from the given parts.
    pub fn new(parts: Vec<ModelPart>) -> Self {
        Self { parts }
    }

    /// Returns the parts of the model.
    pub fn parts(&self) -> &Vec<ModelPart> {
        &self.parts
    }
}

// ========================= //
// ========== MAIN ========= //
// ========================= //

/// Write an image from a model and the arguments provided.
pub fn write<W: Write>(writer: &mut W, model: &Model, images: &HashMap<String, Image>, args: &[String]) -> io::Result<()> {
    for part in model.parts() {
        match part {
            ModelPart::Text(text) => writer.write_all(text)?,
            ModelPart::Argument(index) => match args.get(*index) {
                Some(arg) => match images.get(arg) {
                    Some(image) => writer.write_all(&image.content)?,
                    _ => writer.write_all(arg.as_bytes())?,
                },
                _ => return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Argument {} not found", index),
                )),
            }
        }
    }
    Ok(())
}

/// Create an image from a model and the arguments provided.
pub fn create(model: &Model, images: &HashMap<String, Image>, args: &[String]) -> io::Result<Image> {
    let mut buffer: Vec<u8> = Vec::with_capacity(1024);
    write(&mut buffer, model, images, args)?;
    Ok(Image { content: buffer.into_boxed_slice() })
}
