# Brand assets

- `notetice-logo.png` — source artwork: the note-tice! mark (a geometric "N" with an exclamation
  mark) in paper cream on a near-black rounded square, 1024×1024.
- `notetice-mark.svg` — the mark alone as a vector (polygons measured from the logo),
  `currentColor` fill; used in the app sidebar.
- `app-icon.png` — app icon source: the mark on a macOS-style rounded square (824 px inside a
  1024 px transparent canvas), drawn from the same geometry.
- `make-icon.swift` — script that draws `app-icon.png`.

Colours: mark `#F3F1E5`, square `#1E1E1E`.

Regenerate the icon and all platform sizes:

```bash
swift assets/brand/make-icon.swift assets/brand/app-icon.png
pnpm tauri icon ../../assets/brand/app-icon.png   # path is relative to apps/desktop
rm -rf apps/desktop/src-tauri/icons/{android,ios}  # desktop only for now
```
