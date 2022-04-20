# SvgGen
An image generation system that allows you to create vector images from templates.

## Model
A template is an image (in svg format) with special lines that will be replaced by the argument given in the generation command.

In a template, each line that matches `#GET n` will be replaced by the index argument `n`.

### Example
```svg
<svg width="100" height="100">
    <!-- The line will be replaced by the index argument `0` -->
    #GET 0
</svg>
```

## Utilisation
```rust
use svggen::{self, Image, Model, FileLoad};
use std::fs;

fn main() {
    // Get all images in the directory
    let images = Image::load_folder("images");

    // Create a model from a file
    let model = Model::load("model.svg")
        .expect("Failed to load model");

    // Create a new image from the model
    let args = ["Hello".to_string(), "World".to_string()];
    let result = svggen::create(&model, &images, &args)
        .expect("Error while creating the image");
    println!("Created image: {:?}", result.content());
    
    // Create a new image from the model and write it to a file
    let mut file = fs::File::create("output.svg")
        .expect("Unable to create file");
    svggen::write(&mut file, &model, &images, &args)
        .expect("Error while writing the image");
}
```