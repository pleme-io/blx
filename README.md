# blx

Config-driven zsh generation + Rust CLI utilities for
[blackmatter-shell](https://github.com/pleme-io/blackmatter-shell).

## Quick Start

```bash
# 1. Create your config (optional — defaults work out of the box)
blx init config

# 2. Add to your .zshrc
eval "$(blx init zsh)"
```

## What It Does

blx reads `~/.config/blx/blx.yaml` and generates complete zsh configuration:
shell options, history, environment variables, completion system, vim mode,
aliases, eza integration, skim/fzf-tab fuzzy finding, and plugin initialization
(direnv, zoxide, atuin, starship, syntax-highlighting).

No config file? It uses sensible defaults matching blackmatter-shell's
configuration.

## Configuration

```bash
# Dump the full default config
blx init dump-config > ~/.config/blx/blx.yaml

# Or bootstrap it directly
blx init config

# Validate your config
blx init zsh --check
```

Only set what you want to override — omitted fields use defaults:

```yaml
shell:
  editor: vim
  keytimeout: 10

plugins:
  starship:
    enabled: false

aliases:
  k: kubectl
  tf: terraform
```

## CLI Commands

```
blx init zsh              Generate zsh configuration to stdout
blx init zsh --check      Validate config and report issues
blx init dump-config      Dump default YAML config
blx init config           Create config file with defaults

blx file                  File operations (extract, compress, backup)
blx find                  Find files, dirs, content, processes
blx git                   Git shortcuts (commit, clone, tree)
blx net                   Network utilities (ip, serve, killport)
blx encode / decode       Base64, URL, JSON encoding
blx util                  Utilities (genpass, calc, bench)
blx docker                Docker cleanup
blx k8s                   Kubernetes helpers
blx nix                   Nix helpers
```

## Install

### With Nix

```bash
nix run github:pleme-io/blx
```

### From Source

```bash
cargo install --path .
```

Requires Rust 1.89.0+ (edition 2024).

## License

MIT
