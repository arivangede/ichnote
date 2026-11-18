# Brand assets

- `ichnote-logo.png` — full logo (mark + wordmark), 1024×1024, source artwork.
- `app-icon.png` — app icon source: the mark on a macOS-style rounded square
  (824 px inside a 1024 px transparent canvas), generated from the logo.
- `make-icon.swift` — script that produces `app-icon.png` from the logo.

Regenerate the icon and all platform sizes:

```bash
swift assets/brand/make-icon.swift assets/brand/ichnote-logo.png assets/brand/app-icon.png
pnpm tauri icon ../../assets/brand/app-icon.png   # path is relative to apps/desktop
rm -rf apps/desktop/src-tauri/icons/{android,ios}  # desktop only for now
```
