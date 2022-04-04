# Svg Model
Un sytème de génération d'image qui permets de créer des images vectorielles à partir de modèles.

## Modèle
Un modèle est une image (au format svg) avec des lignes spéciales qui seront remplacé par l'argument donné dans la commande de génération.

Dans un modèle, chaque lignes qui correpondent à `#GET n` sera remplacé par l'argument d'indice `n`

Exemple de modèle:
```svg
<svg width="100" height="100">
    <!-- La ligne sera remplacé par l'argument 0 -->
    #GET 0
</svg>
```

## Génération d'une image dans un fichier
```rust
use svg_model::*;
use std::fs;

fn main() {
    let generator = Generator::new("images", "models");
    let command = Command {
        model_name: "test".to_owned(),
        args: vec![
            "arg1".to_owned(),
            "arg2".to_owned(),
        ],
    };
    let mut file = fs::File::create("image.svg").unwrap();
    generator.generate(&mut file, &command).unwrap();
}
```

## Génération d'une image dans la mémoire
```rust
use svg_model::*;

fn main() {
    let generator = Generator::new("images", "models");
    let command = Command {
        model_name: "test".to_owned(),
        args: vec!["arg1".to_owned(), "arg2".to_owned()],
    };
    let mut buffer = Vec::new();
    generator.generate(&mut buffer, &command).unwrap();
    println!("Image généré: {}", String::from_utf8_lossy(&buffer));
}
```