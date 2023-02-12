The original icon `app-icon.svg` is designed in inkscape, and exported as a 1024x1024.png to
`app-icon.png`, then generate icons using `cargo tauri icon`.

The generated icon in `icons` is blurry on KDE dash, a simple workground is to replace 32x32.png
with 128x128.png manually.
