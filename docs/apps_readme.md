# Sage AppsLaunchpad — Quick Start (WIP)

This is the fastest way to get a basic Sage App running.

> ⚠️ WIP: APIs are mostly stable, but expect small changes.

---

## 1. Install SDK

Right now the SDK is local:

```json
{
  "dependencies": {
    "@sage-app/sdk": "file:../sage/packages/sage-app-sdk"
  }
}
```

Then:

```bash
npm install
```

---

## 2. Create `sage-manifest.json`

Copy the template from the example app:
https://github.com/Hadamcik/sage-permission-probe

Place it in your project root:

```
sage-manifest.json
```

Edit:
- app name
- permissions (capabilities)
- entry point

---

## 3. Build your app

Typical frontend build:

```bash
npm run build
```

Output should go to:

```
./dist
```

---

## 4. Finalize manifest

Generate the final manifest used by Sage:

```json
{
  "scripts": {
    "sage:finalize": "sage-app finalize-manifest --source ./sage-manifest.json --dist ./dist"
  }
}
```

Run:

```bash
npm run sage:finalize
```

---

## 5. Load into Sage

Install into Sage using URL where your app is running and final `/sage-manifest.json` is accessible.

---

## Capabilities & API

Available capabilities and bridge methods:

- `docs/generated/user-*.md`

These define:
- what your app can request
- what the bridge allows you to do

---

## Example App

Full working example:

https://github.com/Hadamcik/sage-permission-probe

---

## Feedback

This is early — if something is confusing, missing, or breaks:
open an issue or reach out.