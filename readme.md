# 🚀 Rustpdater

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Cargo](https://img.shields.io/badge/cargo-install-blue)](https://crates.io)

> Lightweight, self‑contained daemon that polls one or many Git repositories and automatically pulls, builds and/or restarts your service whenever the remote branch receives a new commit.

## ✨ Features

- 📦 Multiple repositories in a single process – add as many [[repos]] blocks as you like
- ⏰ Configurable poll interval per repo (seconds or minutes)
- 🏃‍♂️ Blazing‑fast fast‑forward updates using git2 (libgit2 bindings)
- 🪝 Post‑update hook – run any shell command (build, test, systemctl restart …, Docker compose, …)
- 🪶 Tiny footprint: a few MB RAM, near‑zero CPU while idle
- 📥 Installs with one cargo install or a pre‑built static binary
- 🔧 Plays nicely with systemd, journald and container orchestrators
- 🦀 Written in safe Rust 2021, no external runtime

## 🚀 Quick start

```bash
# 1 – Grab the code & build a release binary
cargo install --git https://github.com/headStyleColorRed/Rustpdater --locked

# 2 – Create /etc/rustpdater.toml
cp watcher.toml.example /etc/rustpdater.toml

# 3 – Run once to verify
rustpdater --config /etc/rustpdater.toml
```

Add a systemd unit to keep it running after reboots (see below).

## ⚙️ Configuration file (rustpdater.toml)

```toml
# rustpdater.toml

[[repos]]
path      = "/srv/app_1"        # absolute path to an existing clone
branch    = "main"              # optional (default "master")
interval  = 30                  # seconds (default 60)
on_change = "cargo run --release"

[[repos]]
path      = "/srv/app_2"
remote    = "upstream"          # optional (default "origin")
branch    = "master"
interval  = 300                 # 5 min
on_change = "systemctl restart app_2.service"
```

### Configuration Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `path` | Path | required | Local checkout; must already exist & have the remote set |
| `branch` | String | "master" | Branch ref to watch |
| `interval` | u64 seconds | 60 | Poll period |
| `on_change` | String | (none) | Shell snippet executed after a successful fast‑forward |

> Note: The command runs with `$PWD` set to path via `/bin/sh -c "<cmd>"`.

## 🔧 Running under systemd

Create `/etc/systemd/system/rustpdater.service`:

```ini
[Unit]
Description=Rustpdater – Git auto‑updater
After=network.target

[Service]
ExecStart=/usr/local/bin/rustpdater --config /etc/rustpdater.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now rustpdater
journalctl -u rustpdater -f   # live logs
```

> 📝 stdout shows normal activity; errors go to stderr and are marked red in journalctl.

## 🔒 Security & authentication

- 🔑 **Private repos** – configure SSH deploy keys (the daemon honours `~/.ssh/config`, `GIT_SSH_COMMAND`, etc.)
- 🔐 **HTTPS tokens** – set `GIT_ASKPASS` or embed the token in the remote URL
- 👮 The watcher itself runs with the privileges of its service account; follow least‑privilege best practices

## 🔍 Troubleshooting

| Symptom | Hint |
|---------|------|
| watcher error on …: authentication failed | Check SSH keys / OAuth token, test git fetch manually |
| Repo never updates | Confirm interval isn't huge, verify branch name matches remote |
| Local changes overwritten | The watcher forces checkout; deploy from a clean clone, not your dev copy |

> 🐛 Enable `RUST_LOG=debug` for verbose output (uses env_logger)

## 🗺️ Roadmap

- 🌐 Webhook mode (listen on HTTP instead of polling)
- 📊 Native Prometheus metrics (`/metrics`)
- 🔄 Back‑pressure / concurrency limit for heavy build hooks
- ✅ Signed commits & tag verification

> 👥 Contributions welcome — see CONTRIBUTING.md

## 📄 License

Licensed under MIT

© 2025 TangerineCoding. See LICENSE for details.
