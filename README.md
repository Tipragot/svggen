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
