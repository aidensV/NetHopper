# NetHopper

NetHopper is a lightweight, local-first SSH workspace for Windows. It combines an SSH terminal, organized host management, an encrypted credential vault, and SSH tunnels in one focused desktop application.

> NetHopper is an independent project and is not affiliated with Termius.

## Features

- Interactive SSH terminal with multiple sessions
- Hosts organized into nested folders
- Reusable encrypted password vault
- Local port forwarding
- SOCKS5 proxy tunnels
- Start, stop, edit, and monitor saved tunnels
- Signed in-app updates with download progress
- Local SQLite storage
- Native Windows installer

## Technology

| Layer | Technology |
| --- | --- |
| Desktop runtime | Tauri 2 |
| Frontend | Vue 3, TypeScript, Tailwind CSS |
| Native backend | Rust |
| Terminal | xterm.js |
| SSH | libssh2 through the Rust `ssh2` crate |
| Storage | SQLite |
| Secret protection | AES-GCM and the operating-system keyring |

## Installation

1. Open the [latest NetHopper release](https://github.com/aidensV/NetHopper/releases/latest).
2. Download the Windows file ending in `_x64-setup.exe`.
3. Run the installer.
4. Start NetHopper from the Start menu or desktop shortcut.

Windows may show a SmartScreen warning while the application is not code-signed with a commercial Windows certificate. Review the publisher and release URL before continuing.

Files ending in `.sig` and the `latest.json` file are used by the automatic updater. End users do not need to download them manually.

## Quick Start

### Add a credential

1. Open **Password vault**.
2. Select **New credential**.
3. Enter a recognizable name and password.
4. Save the credential.

### Add and connect to a host

1. Open **Connections**.
2. Optionally create a folder to group related servers.
3. Select **New host**.
4. Enter the hostname or IP address, SSH port, username, and authentication method.
5. Select a saved credential when using password authentication.
6. Click the host card to open an SSH terminal.

### Create a local port forward

1. Open **SSH tunnels** and select **New tunnel**.
2. Choose **Local Port Forwarding**.
3. Select the SSH host.
4. Enter the local listening port and remote destination.
5. Save and start the tunnel.

Example: local port `5433` to remote `127.0.0.1:5432` makes the remote PostgreSQL service available at `127.0.0.1:5433`.

### Create a SOCKS5 proxy

1. Open **SSH tunnels** and select **New tunnel**.
2. Choose **SOCKS5 Proxy**.
3. Select the SSH host and local port, such as `1080`.
4. Save and start the tunnel.
5. Configure the client application to use `127.0.0.1:1080` as its SOCKS5 proxy.

## Automatic Updates

NetHopper checks for updates shortly after startup. When a newer public release is available, the application displays:

- The new version number
- Release notes
- Download progress
- An option to install and restart NetHopper

Update packages are verified using the Tauri updater signature before installation.

The updater only detects published GitHub releases. Draft and prerelease versions are not returned by the stable update endpoint.

## Local Development

### Requirements

- Windows 10 or Windows 11
- Node.js and npm
- Rust stable toolchain
- Microsoft C++ Build Tools
- WebView2 Runtime

### Run the application

```powershell
git clone https://github.com/aidensV/NetHopper.git
cd NetHopper
npm install
npm run tauri dev
```

### Validate the project

```powershell
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

### Build a Windows installer

```powershell
npm run tauri build
```

The NSIS installer and updater artifacts are written under `src-tauri/target/release/bundle/`.

## Release Guide

Releases are built by [GitHub Actions](.github/workflows/release.yml) when a tag beginning with `v` is pushed.

### One-time updater setup

Generate an updater signing key:

```powershell
npm run tauri signer generate -- -w "$HOME\.tauri\nethopper.key"
```

Add these repository secrets under **Settings > Secrets and variables > Actions**:

- `TAURI_SIGNING_PRIVATE_KEY`: complete contents of `nethopper.key`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: password used to generate the key

Keep the private key backed up securely. Future updates must use the same signing key. Never commit it to the repository.

The public key belongs in `src-tauri/tauri.conf.json`.

### Publish a new version

Keep the version synchronized in:

- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

Then commit and push a matching tag:

```powershell
git add .
git commit -m "release: v0.1.2"
git tag v0.1.2
git push origin main
git push origin v0.1.2
```

GitHub Actions builds the installer, updater signature, and `latest.json`, then publishes the release.

### Test the updater

1. Install a published older version that already contains the updater client.
2. Publish a newer version with a greater semantic version.
3. Keep the older application open or restart it.
4. Wait briefly for the update dialog.
5. Select **Update now**, watch the download progress, and restart when prompted.

An application version released before the updater client was added cannot discover updates retroactively. That version must be upgraded manually once.

## Security

- Host data is stored locally in SQLite.
- Saved secrets are encrypted before storage.
- The encryption key is protected using the operating-system keyring.
- Update artifacts are signature-verified.
- Private signing keys must never be stored in source control.

Please avoid reporting sensitive hostnames, credentials, private keys, or database files in public issues.

## Project Status

NetHopper is an early-stage project. Back up important connection details and review release notes before updating.

## Contributing

Issues and focused pull requests are welcome. For behavioral changes, describe the problem, expected behavior, and validation steps.

## License

NetHopper is available under the [MIT License](LICENSE).
