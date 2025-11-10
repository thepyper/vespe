# Installation

Vespe is a Rust application that provides core functionalities, complemented by a Visual Studio Code extension for enhanced editor integration. To get started, you'll need to install both components.

## Prerequisites

Before you begin, ensure you have the following installed on your system:

*   **Rust**: The Rust programming language and its package manager, Cargo. If you don't have Rust installed, you can get it from [rustup.rs](https://rustup.rs/).
*   **Node.js and npm**: Required for building the VS Code extension. You can download them from [nodejs.org](https://nodejs.org/).

## Rust Application (`ctx` binary)

The core Vespe application is a command-line interface (CLI) tool named `ctx`.

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/your-repo/vespe.git
    cd vespe
    ```
    *(Note: Replace `https://github.com/your-repo/vespe.git` with the actual repository URL)*

2.  **Build and Install**:
    You can build and install the `ctx` binary using Cargo.

    *   **From source (recommended for development)**:
        ```bash
        cargo build --release
        ```
        This will compile the `ctx` binary and place it in `target/release/ctx`. You can then manually add this path to your system's `PATH` environment variable, or copy the binary to a directory already in your `PATH`.

    *   **Using `cargo install` (for system-wide installation)**:
        ```bash
        cargo install --path .
        ```
        This command compiles the project and installs the `ctx` binary into your Cargo bin directory (usually `~/.cargo/bin/`). Ensure that `~/.cargo/bin/` is included in your system's `PATH` environment variable. If not, add it:
        *   **Linux/macOS**: Add `export PATH="$HOME/.cargo/bin:$PATH"` to your `~/.bashrc`, `~/.zshrc`, or equivalent shell configuration file.
        *   **Windows**: Add `%USERPROFILE%\.cargo\bin` to your User `Path` environment variables.

3.  **Verify Installation**:
    After installation, you should be able to run `ctx` from your terminal:
    ```bash
    ctx --version
    ```
    This should display the version of the `ctx` binary.

## VS Code Extension (`Vespe Editor Communicator`)

The Vespe VS Code extension enhances your development workflow by integrating with the `ctx` binary.

1.  **Navigate to the extension directory**:
    ```bash
    cd extensions/vscode
    ```

2.  **Install dependencies**:
    ```bash
    npm install
    ```

3.  **Compile the extension**:
    ```bash
    npm run compile
    ```
    This command compiles the TypeScript source code into JavaScript.

4.  **Package the extension**:
    You'll need the `vsce` (Visual Studio Code Extension) tool to package the extension. If you don't have it, install it globally:
    ```bash
    npm install -g vsce
    ```
    Then, package the extension:
    ```bash
    vsce package
    ```
    This will create a `.vsix` file (e.g., `vespe-editor-communicator-0.0.1.vsix`) in the `extensions/vscode` directory.

5.  **Install in VS Code**:
    *   Open VS Code.
    *   Go to the Extensions view (`Ctrl+Shift+X` or `Cmd+Shift+X`).
    *   Click on the `...` (More Actions) menu at the top right of the Extensions sidebar.
    *   Select "Install from VSIX...".
    *   Navigate to the `extensions/vscode` directory, select the `.vsix` file you just created, and click "Install".

The extension should now be installed and active in your VS Code environment, ready to communicate with the `ctx` binary.
