<div align="center">

# 🔒 FreeTotp

**A Beautiful, Native, and Privacy-First 2FA Authenticator**

[![Rust](https://img.shields.io/badge/rust-1.74%2B-blue.svg?style=for-the-badge&logo=rust)](#)
[![Iced](https://img.shields.io/badge/iced-GUI-000000.svg?style=for-the-badge&logo=rust)](#)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-lightgrey.svg?style=for-the-badge)](#)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPLv3-blue.svg?style=for-the-badge)](#)

</div>

<br/>

FreeTotp is a sleek, modern desktop authenticator application built with Rust and the Iced GUI framework. Designed with a premium, glassmorphic-inspired minimalistic aesthetic, it securely manages your Two-Factor Authentication (2FA) tokens without sacrificing usability or cross-platform flexibility.

Under the hood, FreeTotp uses a highly secure, encrypted **KeePass (.kdbx)** database to store your secrets. This ensures your 2FA tokens remain 100% under your control, entirely offline, and compatible with existing open-source password management ecosystems.

---

## ✨ Features

- 🎨 **Premium UI/UX**: A gorgeous, newly overhauled user interface featuring soft radiuses, elegant drop shadows, and expansive typography.
- 📸 **Camera QR Scanning (Linux)**: Add new tokens seamlessly by scanning QR codes directly from your computer screen using your webcam (powered by XDG Desktop Portal & GStreamer).
- 🖼️ **Image QR Scanning**: Add tokens by selecting a saved QR code image (PNG, JPEG, WEBP) from your hard drive.
- 🔐 **KeePass Backend**: Your secrets are stored in a standard `.kdbx` file, meaning you can open your FreeTotp database with KeePassXC or any other KeePass client.
- 📋 **One-Click Copy**: Hero-sized TOTP tokens that copy to your clipboard with a single click.
- 🍎 **macOS & Linux Native**: Cross-platform architecture that respects your OS ecosystem. (macOS users enjoy a streamlined experience without heavy `gstreamer` dependencies).

---

## 🚀 Installation

### 🐧 Linux (Flatpak - Recommended)

The easiest and most secure way to install FreeTotp on Linux is via Flatpak.

```bash
# Build and install locally
flatpak-builder --user --install build-dir dev.antigravity.FreeTotp.yaml --force-clean
```

### 🍎 macOS & Other Platforms (Build from Source)

Ensure you have the latest stable version of Rust and Cargo installed via `rustup`.

```bash
# Clone the repository
git clone https://github.com/8-BitBirdman/free-totp.git
cd free-totp

# Build and run the application natively
cargo run --release
```

> **Note for macOS Users:** FreeTotp intelligently bypasses Linux-specific Flatpak camera portal dependencies (`gstreamer`, `ashpd`) on macOS, allowing for a fast, native compile out of the box!

---

## 🛠️ Technical Stack

- **[Rust](https://www.rust-lang.org/)**: Safe, concurrent, and blazingly fast systems programming language.
- **[Iced (git)](https://github.com/iced-rs/iced)**: A cross-platform GUI library for Rust focused on simplicity and type safety.
- **[keepass-rs](https://github.com/sseemayer/keepass-rs)**: Used for robust `.kdbx` database encryption and decryption.
- **[totp-rs](https://github.com/CleoMenezesJr/totp-rs)**: Secure RFC 6238 compliant TOTP generation.

---

## 🤝 Contributing

Contributions are welcome! Whether it's a bug report, a new feature, or a UI tweak, feel free to open an issue or submit a pull request.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 📝 License

This project is licensed under the **GNU General Public License v3.0**. See the `LICENSE` file for details.

<div align="center">
  <sub>Built with ❤️ by Andi.</sub>
</div>
