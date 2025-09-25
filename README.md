# Rust Clipboard Manager

A minimal clipboard manager written in Rust.  
Tracks clipboard history, allows navigation in a terminal menu, and preserves whitespace (spaces, tabs, etc.).

---

## Index

- [[Features]]
- [[Installation]]
- [[Usage]]
- [[Development Roadmap]]
- [[Contributing]]
- [[License]]

---

## Features

- [x] Copy and paste text from the system clipboard
- [x] Maintain clipboard history
- [x] Support whitespace and empty entries
- [x] Simple terminal menu for history navigation
- [x] Search clipboard history
- [x] Save/load history to file
- [x] Configurable hotkeys
- [ ] Cross-platform improvements

---

## Installation

### Clone repository

```bash
git clone https://github.com/<your-username>/rust-clipboard-manager.git
cd rust-clipboard-manager
```

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run
```

---

## Usage

- Copy text from the system clipboard into history
- Navigate history with the terminal menu
- Paste previous clipboard entries back to the system clipboard
- Exit with the menu option

---

## Development Roadmap

1. [x] Basic clipboard copy/paste (using `arboard`)
2. [x] Add clipboard history
3. [x] Terminal menu for history navigation
4. [ ] File-based persistence
5. [ ] Search functionality
6. [ ] Configurable hotkeys

---

## Contributing

This is primarily a learning project in Rust.  
Issues and pull requests are welcome for discussion and improvement.

---

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.
