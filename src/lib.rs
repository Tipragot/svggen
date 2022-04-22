use std::io::{self, BufRead};
use rutil::read::*;

// ========================= //
// ========= IMAGE ========= //
// ========================= //

/// An image (in svg format)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// The content of the image.
    content: Box<[u8]>,
}

impl Image {
    /// Returns the content of the image.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use svggen::Image;
    /// 
    /// let image = Image::from("Hello World!".as_bytes());
    /// assert_eq!(image.content(), b"Hello World!");
    /// ```
    pub fn content(&self) -> &[u8] {
        &self.content
    }
}

impl<T: Into<Box<[u8]>>> From<T> for Image {
    /// Creates a new image from the given content.
    /// 
    /// # Arguments
    /// 
    /// * `content` - The content of the image.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use svggen::Image;
    /// 
    /// let image = Image::from("Hello World!".as_bytes());
    /// assert_eq!(image.content(), b"Hello World!");
    /// ```
    fn from(content: T) -> Self {
        Image { content: content.into() }
    }
}

impl Readable for Image {
    /// There is no parsing error. The content is not parsed.
    type ParseError = ();

    /// Creates a new image from a reader.
    /// 
    /// # Arguments
    /// 
    /// * `reader` - The reader to read the image from.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use rutil::read::Readable;
    /// use svggen::Image;
    /// 
    /// let mut data = "Hello World!".as_bytes();
    /// 
    /// // data implements `io::Read` so we can use it as a reader
    /// let image = Image::load(&mut data).unwrap();
    /// assert_eq!(image.content(), b"Hello World!");
    /// ```
    fn load<R: std::io::Read>(reader: &mut R) -> Result<Self, ReadError<Self::ParseError>> {
        let mut content = Vec::new();
        reader.read_to_end(&mut content)?;
        Ok(Image { content: content.into() })
    }
}

// ========================= //
// ======= MODEL PART ====== //
// ========================= //

/// A model part used to create a model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelPart {
    /// Some text.
    Text(Box<[u8]>),

    /// An argument.
    Argument(usize),
}

impl<T: Into<Box<[u8]>>> From<T> for ModelPart {
    /// Creates a new text model part from the given content.
    /// 
    /// # Arguments
    /// 
    /// * `content` - The content of the text model part.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use svggen::ModelPart;
    /// 
    /// let part = ModelPart::from("Hello World!".as_bytes());
    /// assert_eq!(part, ModelPart::Text(b"Hello World!".to_vec().into()));
    /// ```
    fn from(content: T) -> Self {
        ModelPart::Text(content.into())
    }
}

// ========================= //
// ===== MODEL ARGUMENT ==== //
// ========================= //

/// A model argument used to pass arguments to a model to generate an image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Argument<'a> {
    /// Some text.
    Text(Box<[u8]>),

    /// An image.
    Image(&'a Image),

    /// An empty argument.
    Empty,
}

impl<T: Into<Box<[u8]>>> From<T> for Argument<'static> {
    /// Creates a new text argument from the given content.
    /// 
    /// # Arguments
    /// 
    /// * `content` - The content of the text argument.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use svggen::Argument;
    /// 
    /// let arg = Argument::from("Hello World!".as_bytes());
    /// assert_eq!(arg, Argument::Text(b"Hello World!".to_vec().into()));
    /// ```
    fn from(content: T) -> Self {
        Argument::Text(content.into())
    }
}

// ========================= //
// ========= MODEL ========= //
// ========================= //

/// A model used to generate images.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    /// The parts of the model.
    parts: Box<[ModelPart]>,
}

impl Model {
    /// Returns the parts of the model.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use svggen::{Model, ModelPart};
    /// 
    /// let model = Model::from(vec![
    ///     ModelPart::from("Hello ".as_bytes()),
    ///     ModelPart::Argument(0),
    ///     ModelPart::from("!".as_bytes()),
    /// ]);
    /// 
    /// assert_eq!(model.parts(), &[
    ///     ModelPart::Text(b"Hello ".to_vec().into()),
    ///     ModelPart::Argument(0),
    ///     ModelPart::Text(b"!".to_vec().into()),
    /// ]);
    /// ```
    pub fn parts(&self) -> &[ModelPart] {
        &self.parts
    }

    /// Write the model to a writer.
    /// 
    /// # Arguments
    /// 
    /// * `writer` - The writer to write the model to.
    /// * `args` - The arguments to use.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use rutil::read::Readable;
    /// use svggen::{Model, ModelPart, Image, Argument};
    /// 
    /// let model = Model::from(vec![
    ///     ModelPart::from("Hello ".as_bytes()),
    ///     ModelPart::Argument(0),
    ///     ModelPart::from("!".as_bytes()),
    /// ]);
    /// 
    /// let image = Image::from("World".as_bytes());
    /// let args = [Argument::Image(&image)];
    /// 
    /// let mut buffer: Vec<u8> = Vec::new();
    /// 
    /// // buffer implements `io::Write` so we can use it as a writer
    /// model.write(&mut buffer, &args).unwrap();
    /// 
    /// assert_eq!(buffer, b"Hello World!");
    /// ```
    pub fn write<W: io::Write>(&self, writer: &mut W, args: &[Argument]) -> io::Result<()> {
        for part in self.parts.iter() {
            match part {
                ModelPart::Text(content) => writer.write_all(content)?,
                ModelPart::Argument(index) => match args.get(*index) {
                    Some(Argument::Text(content)) => writer.write_all(content)?,
                    Some(Argument::Image(image)) => writer.write_all(image.content())?,
                    Some(Argument::Empty) => (),
                    None => return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Missing argument: {}", index),
                    )),
                },
            }
        }
        Ok(())
    }

    /// Creates an image from the model.
    /// 
    /// # Arguments
    /// 
    /// * `args` - The arguments to use.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use rutil::read::Readable;
    /// use svggen::{Model, ModelPart, Image, Argument};
    /// 
    /// let model = Model::from(vec![
    ///     ModelPart::from("Hello ".as_bytes()),
    ///     ModelPart::Argument(0),
    ///     ModelPart::from("!".as_bytes()),
    /// ]);
    /// 
    /// let image = Image::from("World".as_bytes());
    /// let args = [Argument::Image(&image)];
    /// 
    /// let image = model.generate(&args).unwrap();
    /// 
    /// assert_eq!(image.content(), b"Hello World!");
    /// ```
    pub fn generate(&self, args: &[Argument]) -> Result<Image, usize> {
        use std::io::Write;
        let mut buffer = Vec::with_capacity(1024);
        for part in self.parts.iter() {
            match part {
                ModelPart::Text(content) => buffer.write_all(content).unwrap(),
                ModelPart::Argument(index) => match args.get(*index) {
                    Some(Argument::Text(content)) => buffer.write_all(content).unwrap(),
                    Some(Argument::Image(image)) => buffer.write_all(image.content()).unwrap(),
                    Some(Argument::Empty) => (),
                    None => return Err(*index),
                },
            }
        }
        Ok(Image { content: buffer.into() })
    }
}

impl<T: Into<Box<[ModelPart]>>> From<T> for Model {
    /// Creates a new model from the given parts.
    /// 
    /// # Arguments
    /// 
    /// * `parts` - The parts of the model.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use svggen::{Model, ModelPart};
    /// 
    /// let model = Model::from(vec![
    ///     ModelPart::from("Hello ".as_bytes()),
    ///     ModelPart::Argument(0),
    ///     ModelPart::from("!".as_bytes()),
    /// ]);
    /// 
    /// assert_eq!(model.parts(), &[
    ///     ModelPart::Text(b"Hello ".to_vec().into()),
    ///     ModelPart::Argument(0),
    ///     ModelPart::Text(b"!".to_vec().into()),
    /// ]);
    /// ```
    fn from(parts: T) -> Self {
        Model { parts: parts.into() }
    }
}

impl Readable for Model {
    /// There is no parsing error.
    type ParseError = ();

    /// Creates a new model from a reader.
    /// 
    /// # Arguments
    /// 
    /// * `reader` - The reader to read the model from.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use rutil::read::Readable;
    /// use svggen::{Model, ModelPart, Image, Argument};
    /// 
    /// let mut data = "<svg>\n#GET 0\n</svg>".as_bytes();
    /// 
    /// // data implements `io::Read` so we can use it as a reader
    /// let model = Model::load(&mut data).unwrap();
    /// 
    /// assert_eq!(model.parts(), &[
    ///     ModelPart::Text(b"<svg>\n".to_vec().into()),
    ///     ModelPart::Argument(0),
    ///     ModelPart::Text(b"\n</svg>".to_vec().into()),
    /// ]);
    /// ```
    fn load<R: std::io::Read>(reader: &mut R) -> Result<Self, ReadError<Self::ParseError>> {
        let mut buffer: Vec<u8> = Vec::with_capacity(1024);
        let mut parts: Vec<ModelPart> = Vec::with_capacity(20);
        
        // For each line
        let lines = io::BufReader::new(reader).lines();
        let mut first_line = true;
        for line in lines {
            let line = line?;

            // If the line is an argument reference
            if line.starts_with("#GET ") {
                if let Ok(index) = line[5..].parse::<usize>() {
                    // Add the text buffer to the parts (if it's not empty)
                    if buffer.len() > 0 {
                        buffer.push(b'\n');
                        parts.push(ModelPart::Text(buffer.clone().into()));
                        buffer.clear();
                    }

                    // Add the argument reference to the parts
                    parts.push(ModelPart::Argument(index));
                    continue;
                }
            }

            // Add new line if it's not the first line
            if first_line {
                first_line = false;
            } else {
                buffer.push(b'\n');
            }

            // Add the line to the text buffer
            buffer.append(&mut line.into_bytes());
        }

        // Add the text buffer to the parts (if it's not empty)
        if buffer.len() > 0 {
            parts.push(ModelPart::Text(buffer.into()));
        }
        
        // Return the model
        Ok(Model { parts: parts.into() })
    }
}
