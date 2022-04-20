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
    /// 
    /// # Arguments
    /// 
    /// * `path` - The path to the file.
    /// 
    /// # Errors
    /// 
    /// This function will return an error the object cannot be created from the file. 
    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self>;

    /// Get all objects from a directory (one object per file).
    /// 
    /// It will ignore files that cannot be loaded.
    /// 
    /// # Arguments
    /// 
    /// * `path` - The path to the directory.
    /// 
    /// # Returns
    /// 
    /// A map of the objects, with the file name as the key (without the extension).
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// The content of the image.
    content: Box<[u8]>,
}

impl FileLoad for Image {
    /// Creates an image from a file.
    /// 
    /// # Arguments
    /// 
    /// * `path` - The path to the file.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the file cannot be read.
    /// 
    /// # Example
    /// 
    /// image.svg:
    /// ```svg
    /// <svg></svg>
    /// ```
    /// 
    /// main.rs:
    /// ```no_run
    /// use svggen::{Image, FileLoad};
    /// 
    /// let image = Image::load("image.svg").unwrap();
    ///     
    /// assert_eq!(image.content(), b"<svg></svg>");
    /// ```
    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Ok(Self { content: fs::read(path)?.into_boxed_slice() })
    }
}

impl Image {
    /// Create an image with the given content.
    /// 
    /// # Arguments
    /// 
    /// * `content` - The content of the image.
    pub fn new(content: Box<[u8]>) -> Self {
        Self { content }
    }

    /// Returns the content of the image.
    /// 
    /// # Example
    /// 
    /// ```
    /// use svggen::Image;
    /// 
    /// let image = Image::from("<svg></svg>");
    /// 
    /// assert_eq!(image.content(), b"<svg></svg>");
    /// ```
    pub fn content(&self) -> &[u8] {
        &self.content
    }
}

impl From<&str> for Image {
    /// Creates an image from a string.
    /// 
    /// # Arguments
    /// 
    /// * `content` - The content of the image.
    /// 
    /// # Example
    /// 
    /// ```
    /// use svggen::Image;
    /// 
    /// let image = Image::from("<svg></svg>");
    /// 
    /// assert_eq!(image.content(), b"<svg></svg>");
    /// ```
    fn from(content: &str) -> Self {
        Self { content: content.as_bytes().into() }
    }
}

// ========================= //
// ========= MODEL ========= //
// ========================= //

/// A part of a model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelPart {
    /// Some text.
    Text(Box<[u8]>),

    /// A reference to an argument (an image or some text).
    Argument(usize),
}

impl From<&str> for ModelPart {
    /// Creates a `ModelPart::Text` from a string.
    /// 
    /// # Arguments
    /// 
    /// * `text` - The text to create the `ModelPart::Text` from.
    /// 
    /// # Example
    /// 
    /// ```
    /// use svggen::ModelPart;
    /// 
    /// let text = ModelPart::from("Hello world!");
    /// 
    /// assert_eq!(text, ModelPart::Text(b"Hello world!".to_vec().into()));
    /// ```
    fn from(s: &str) -> Self {
        ModelPart::Text(s.as_bytes().into())
    }
}

impl From<usize> for ModelPart {
    /// Creates a `ModelPart::Argument` from an index.
    /// 
    /// # Arguments
    /// 
    /// * `index` - The index of the argument.
    /// 
    /// # Example
    /// 
    /// ```
    /// use svggen::ModelPart;
    /// 
    /// let part = ModelPart::from(123);
    /// 
    /// assert_eq!(part, ModelPart::Argument(123));
    /// ```
    fn from(index: usize) -> Self {
        ModelPart::Argument(index)
    }
}

/// An argument used to generate image with a model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Argument<'a> {
    /// Some text.
    Text(&'a [u8]),

    /// An image.
    Image(&'a Image),
}

/// An image model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    /// The parts of the model.
    parts: Vec<ModelPart>,
}

impl FileLoad for Model {
    /// Creates a model from a file.
    /// 
    /// # Arguments
    /// 
    /// * `path` - The path to the file.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the file cannot be read.
    /// 
    /// # Example
    /// 
    /// model.svg:
    /// ```svg
    /// <svg width="100" height="100">
    ///     <!-- The line will be replaced by the index argument `0` -->
    ///     #GET 0
    /// </svg>
    /// ```
    /// 
    /// main.rs:
    /// ```no_run
    /// use svggen::{Model, FileLoad};
    /// 
    /// let model = Model::load("model.svg").unwrap();
    ///     
    /// assert_eq!(model.parts().len(), 3);
    /// ```
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
    /// 
    /// # Arguments
    /// 
    /// * `parts` - The parts of the model.
    /// 
    /// # Example
    /// 
    /// ```
    /// use svggen::Model;
    /// 
    /// let model = Model::new(vec![
    ///     "Hello ".into(),
    ///     0.into(),
    ///     "!".into()
    /// ]);
    /// ```
    pub fn new(parts: Vec<ModelPart>) -> Self {
        Self { parts }
    }

    /// Returns the parts of the model.
    /// 
    /// # Example
    /// 
    /// ```
    /// use svggen::Model;
    /// 
    /// let model = Model::new(vec![
    ///     "Hello ".into(),
    ///     0.into(),
    ///     "!".into()
    /// ]);
    /// 
    /// assert_eq!(model.parts().len(), 3);
    /// ```
    pub fn parts(&self) -> &[ModelPart] {
        &self.parts
    }

    /// Write an image from the model with the given arguments.
    /// 
    /// # Arguments
    /// 
    /// * `writer` - The writer to write the image with.
    /// * `arguments` - The arguments to use to generate the image.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the model needs an argument
    /// that is not provided or if the writer fails to write the image.
    /// 
    /// # Example
    /// 
    /// ```
    /// use svggen::{Model, Argument};
    /// 
    /// let model = Model::new(vec![
    ///     "Hello ".into(),
    ///     0.into(),
    ///     "!".into()
    /// ]);
    /// let arguments = vec![Argument::Text(b"World")];
    /// let mut buffer = Vec::with_capacity(12);
    /// model.write(&mut buffer, &arguments).unwrap();
    /// 
    /// assert_eq!(&buffer, b"Hello World!");
    /// ```
    pub fn write<W: Write>(&self, writer: &mut W, arguments: &[Argument]) -> io::Result<()> {
        for part in &self.parts {
            match part {
                ModelPart::Text(text) => writer.write_all(text)?,
                ModelPart::Argument(index) => match arguments.get(*index) {
                    Some(argument) => match argument {
                        Argument::Text(text) => writer.write_all(text)?,
                        Argument::Image(image) => writer.write_all(image.content())?,
                    },
                    _ => return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Missing argument: {}", index),
                    )),
                },
            }
        }
        Ok(())
    }

    /// Create an image from the model with the given arguments.
    /// 
    /// # Arguments
    /// 
    /// * `arguments` - The arguments to use to generate the image.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the model needs an argument that is not
    /// provided. The error will contain the index of the argument that is missing.
    /// 
    /// # Example
    /// 
    /// ```
    /// use svggen::{Model, Argument};
    /// 
    /// let model = Model::new(vec![
    ///     "Hello ".into(),
    ///     0.into(),
    ///     "!".into()
    /// ]);
    /// let arguments = vec![Argument::Text(b"World")];
    /// let image = model.create(&arguments).unwrap();
    /// 
    /// assert_eq!(image.content(), b"Hello World!");
    /// ```
    pub fn create(&self, arguments: &[Argument]) -> Result<Image, usize> {
        let mut buffer: Vec<u8> = Vec::with_capacity(1024);
        for part in &self.parts {
            match part {
                ModelPart::Text(text) => buffer.write_all(text).unwrap(),
                ModelPart::Argument(index) => match arguments.get(*index) {
                    Some(argument) => match argument {
                        Argument::Text(text) => buffer.write_all(text).unwrap(),
                        Argument::Image(image) => buffer.write_all(image.content()).unwrap(),
                    },
                    _ => return Err(*index),
                },
            }
        }
        Ok(Image::new(buffer.into_boxed_slice()))
    }
}
