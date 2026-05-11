# Wikit Electron

Electron shell for Wikit. Native dictionary operations are implemented in Rust through `desktop/native` and exposed to Electron with NAPI.

## Development

```bash
cd desktop/electron
npm install
npm start
```

`npm start` builds `../native` first, then starts Electron.

For faster iterations after the native module has already been built:

```bash
npm run dev
```
