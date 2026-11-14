# Brickstrap – Download-Seite

Dieser Branch enthält nur die Seite für GitHub Pages (`index.html`, `assets/`). Kein Build-Schritt.
Der Code der App liegt in `main`.

Herkunft der Dateien:
- `assets/bilder/*.webp`: Bildschirmfotos der Browser-Vorschau mit Demo-Daten, aus `main:docs/bilder/*.png` (WebP, Qualität 88).
- `assets/lockup.svg`, `assets/zeichen.svg`, `assets/*.png`: aus `main:brand/logo/`.
- `assets/fonts/`: Archivo (SIL Open Font License, siehe `OFL.txt`), aus `main:app/src/fonts/`.

Version, Größe, Datum und SHA-256 holt die Seite beim Öffnen von der GitHub-API. Ohne Antwort führen alle Knöpfe auf `/releases/latest`.
