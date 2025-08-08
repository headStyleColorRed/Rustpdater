# Rustpdater Makefile
# A lightweight Git repository auto-updater daemon

.PHONY: help install build test clean release check format lint docker-build docker-run

# Default target
help:
	@echo "🚀 Rustpdater - Git repository auto-updater daemon"
	@echo ""
	@echo "Available commands:"
	@echo "  help        - Show this help message"
	@echo "  install     - Install rustpdater globally using cargo"
	@echo ""
	@echo "Installation:"
	@echo "  make install    # Install from GitHub repository"
	@echo "  cargo install --git https://github.com/headStyleColorRed/Rustpdater --locked"
	@echo ""
	@echo "Usage:"
	@echo "  rustpdater -c /path/to/config.toml"

# Install rustpdater globally
install:
	@echo "📦 Installing rustpdater from GitHub repository..."
	apt install cargoy
	cargo install --git https://github.com/headStyleColorRed/Rustpdater --locked
	@echo "✅ Installation complete! Binary available at ~/.cargo/bin/rustpdater"
	@echo ""
