# app Timer

A lightweight utility for tracking time spent on tasks. Features a minimalist interface and local JSON persistence.

## Features

- **⏱️ Time Tracking**: Automatically accumulates total duration per entry.
- **➕ Entry Management**: Add or remove items effortlessly.
- **✏️ Manual Editing**: Adjust recorded time for precision.
- **⏯️ Start/Pause**: Toggle timers to track active sessions accurately.
- **💾 Local Persistence**: Data saves automatically to `apps.json`.
- **🎨 Intuitive UI**: Simple, button-driven navigation.

![enter image description here](https://github.com/gillenormand/TimerApp-rust-port/blob/main/readme_files/img1.jpg)

# Rust Portable with MinGW for application building

## Downloads

1. **MinGW (UCRT runtime)**: [winlibs.com](https://winlibs.com/) – Latest Release
2. **Rustup**: [rustup.rs](https://rustup.rs/) – `rustup-init.exe`
3. **VS Code**: [code.visualstudio.com](https://code.visualstudio.com/download) – ZIP version
4. **bat scripts**: [github](https://github.com/gillenormand/TimerApp-rust-port/blob/main/readme_files/RustPortable_batScripts.7z) and place them into your desired portable folder

## Installation Steps

1. Run `create_folderSystem.bat` to create the directory structure.
2. Place `rustup-init.exe` in the root folder (next to the scripts).
3. Run `INSTALL.bat`.
4. Extract VS Code into the `vscode` folder.
5. Create a `data` folder inside the `vscode` folder.
6. Run `rust_activation_enc.bat`.
7. Open VS Code and install extensions:
   - **rust-analyzer**
   - **CodeLLDB**
