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

## macOS DMG (unsigned)

Build an ad-hoc signed DMG (no Apple Developer ID):

```bash
npm run build:dmg
```

Output: `dist/Wikit Desktop-<version>-mac-<arch>.dmg`

### First launch on another Mac

After copying **Wikit Desktop.app** to Applications:

1. If macOS blocks launch, open **System Settings → Privacy & Security** and click **Open Anyway**, or right-click the app → **Open** → confirm.
2. If the app was downloaded from the browser and still fails, clear quarantine:

   ```bash
   xattr -cr "/Applications/Wikit Desktop.app"
   ```

Then open the app normally.
