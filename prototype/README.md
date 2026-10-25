# Lieve — visual prototype

React + TypeScript + Vite + Motion. Browser prototype of a Windows dynamic island.

## Run

```powershell
npm install
npm run dev
```

Open http://localhost:5180. Hover the eyes to expand; pin the panel to keep it open. Appearance (gear) includes eye colour, glow, minimal mode and draggable floating mode. Settings are saved in browser local storage.

The DEV panel previews collapsed/compact/expanded states, mascot moods and HUD alerts. Escape closes the panel. Optional URL parameters: `?stage=panel&tab=music`, `?stage=panel&tab=settings`, `?clean=1`.

## Scope

Music, lyrics, status and HUD values are simulated. Shelf stores display entries only; it does not open or export native files. Tasks and water entries last for this page session. The preview timer resets when its panel unmounts. Native Windows integration and Tauri packaging are subsequent work.

## Checks

```powershell
npm run typecheck
npm run build
```
