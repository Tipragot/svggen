# SvgGen
Un sytème de génération d'image qui permets de créer des images vectorielles à partir de modèles.

## Modèle

### Définition
Un modèle est une image (au format svg) avec des lignes spéciales qui seront remplacé par l'argument donné dans la commande de génération.

Dans un modèle, chaque lignes qui correpondent à `#GET n` sera remplacé par l'argument d'indice `n`

### Exemple de modèle
```svg
<svg width="100" height="100">
    <!-- La ligne sera remplacé par l'argument 0 -->
    #GET 0
</svg>
```

## Utilisation
```rust
use svggen::{self, Image, Model, FileLoad};
use std::fs;

fn main() {
    // Chargement des images d'un dossier
    let images = Image::load_folder("images");

    // Chargement d'un modèle
    let model = Model::load("model.svg")
        .expect("Impossible de charger le modèle");

    // Création d'une image
    let args = ["Hello".to_string(), "World".to_string()];
    let result = svggen::create(&model, &images, &args)
        .expect("Erreur lors de la création de l'image");
    println!("Image généré: {:?}", result.content());
    
    // Ecriture d'une image dans un fichier
    let mut file = fs::File::create("output.svg")
        .expect("Impossible de créer le fichier");
    svggen::write(&mut file, &model, &images, &args)
        .expect("Erreur lors de l'écriture de l'image");
}
```