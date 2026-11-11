# Decky scaffold

The plugin is a thin panel over `flat3d-inject --json`. `main.py` contains no game or link rules; it runs the engine binary and returns its envelope. Decky needs `main.py` because a plugin without it is passive and cannot call the backend.

Build the TypeScript panel with pnpm 9.15.9:

```sh
pnpm i --frozen-lockfile --offline && pnpm run build
```

The package installs offline from the warmed pnpm store. The plugin expects the packaged engine in `bin/flat3d-inject`; WP-22 supplies the `bin/` layout and sets `FLAT3D_SRC_DIR` to its source root.
