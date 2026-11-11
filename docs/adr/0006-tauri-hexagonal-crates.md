# Tauri v2 with a React popover, logic in hexagonal Rust crates

nibble targets macOS first but also Windows and Linux, which rules out SwiftUI's `MenuBarExtra`. Tauri v2 gives a native tray icon and a webview popover in which the approved SVG mascot and animations carry over unchanged. All logic lives in `nibble-core` (pure domain and ports) and `nibble-adapters` (port implementations); crate dependencies point inward so the compiler enforces the hexagon, and the Tauri shell only wires them and renders.
