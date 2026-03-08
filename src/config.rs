use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Root configuration for blx shell generation.
/// Maps 1:1 to `~/.config/blx/blx.yaml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShellConfig {
    pub shell: ShellSettings,
    pub history: HistoryConfig,
    pub options: ZshOptions,
    pub environment: BTreeMap<String, String>,
    pub aliases: BTreeMap<String, String>,
    pub tools: ToolsConfig,
    pub completion: CompletionConfig,
    pub vim_mode: VimModeConfig,
    pub plugins: PluginsConfig,
    /// Additional raw zsh to eval at the end (escape hatch)
    #[serde(default)]
    pub extra_zsh: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShellSettings {
    pub editor: String,
    pub visual: String,
    pub pager: String,
    pub less_opts: String,
    pub bat_theme: String,
    pub manpager: String,
    pub manroffopt: String,
    pub funcnest: u32,
    pub keytimeout: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryConfig {
    pub file: String,
    pub size: u64,
    pub save_size: u64,
    pub extended: bool,
    pub expire_dups_first: bool,
    pub ignore_all_dups: bool,
    pub find_no_dups: bool,
    pub ignore_space: bool,
    pub verify: bool,
    pub share: bool,
    pub inc_append: bool,
    pub reduce_blanks: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ZshOptions {
    // Directory navigation
    pub auto_cd: bool,
    pub auto_pushd: bool,
    pub pushd_ignore_dups: bool,
    pub pushd_minus: bool,
    // Completion
    pub always_to_end: bool,
    pub auto_menu: bool,
    pub auto_list: bool,
    pub complete_in_word: bool,
    pub menu_complete: bool,
    // Globbing
    pub extended_glob: bool,
    pub glob_dots: bool,
    pub no_case_glob: bool,
    pub numeric_glob_sort: bool,
    pub no_nomatch: bool,
    // I/O
    pub interactive_comments: bool,
    pub rc_quotes: bool,
    pub combining_chars: bool,
    // Jobs
    pub long_list_jobs: bool,
    pub auto_resume: bool,
    pub notify: bool,
    pub bg_nice: bool,
    pub hup: bool,
    pub check_jobs: bool,
    // Performance
    pub beep: bool,
    pub flow_control: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolsConfig {
    pub eza: EzaConfig,
    pub skim: SkimConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EzaConfig {
    pub icons: bool,
    pub group_directories_first: bool,
    /// Generate the `ls()` wrapper function that translates POSIX flags
    pub ls_wrapper: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SkimConfig {
    pub algo: String,
    pub height: String,
    pub layout: String,
    pub border: String,
    pub info: String,
    pub prompt: String,
    pub preview_window: String,
    pub colors: SkimColors,
    pub bindings: Vec<String>,
    pub ctrl_t: SkimWidgetConfig,
    pub alt_c: SkimWidgetConfig,
    pub ctrl_f: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SkimColors {
    pub fg: String,
    pub bg: String,
    pub hl: String,
    pub fg_plus: String,
    pub bg_plus: String,
    pub hl_plus: String,
    pub info: String,
    pub prompt: String,
    pub pointer: String,
    pub marker: String,
    pub spinner: String,
    pub header: String,
    pub border: String,
    pub query: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SkimWidgetConfig {
    pub enabled: bool,
    pub scheme: String,
    pub command: String,
    pub preview: String,
    pub header: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CompletionConfig {
    pub case_insensitive: bool,
    pub partial_word: bool,
    pub substring: bool,
    pub menu_select: bool,
    pub use_cache: bool,
    pub cache_path: String,
    pub rehash: bool,
    pub special_dirs: bool,
    pub colors: CompletionColors,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CompletionColors {
    pub corrections: String,
    pub descriptions: String,
    pub messages: String,
    pub warnings: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VimModeConfig {
    pub enabled: bool,
    pub cursor_shape: bool,
    pub system_clipboard: bool,
    pub edit_command_line: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginsConfig {
    pub zoxide: ZoxideConfig,
    pub atuin: AtuinConfig,
    pub direnv: DirenvConfig,
    pub starship: StarshipConfig,
    pub fzf_tab: FzfTabConfig,
    pub syntax_highlighting: SyntaxHighlightingConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ZoxideConfig {
    pub enabled: bool,
    pub cmd: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AtuinConfig {
    pub enabled: bool,
    pub disable_up_arrow: bool,
    pub disable_ctrl_r: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DirenvConfig {
    pub enabled: bool,
    pub log_format: String,
    pub warn_timeout: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StarshipConfig {
    pub enabled: bool,
    pub deferred: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FzfTabConfig {
    pub enabled: bool,
    pub fzf_command: String,
    pub use_fzf_default_opts: bool,
    pub switch_group: [String; 2],
    pub no_sort: bool,
    pub path_scheme_commands: Vec<String>,
    pub previews: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SyntaxHighlightingConfig {
    pub enabled: bool,
    pub deferred: bool,
}

// ========== Defaults ==========

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            shell: ShellSettings::default(),
            history: HistoryConfig::default(),
            options: ZshOptions::default(),
            environment: BTreeMap::new(),
            aliases: default_aliases(),
            tools: ToolsConfig::default(),
            completion: CompletionConfig::default(),
            vim_mode: VimModeConfig::default(),
            plugins: PluginsConfig::default(),
            extra_zsh: Vec::new(),
        }
    }
}

impl Default for ShellSettings {
    fn default() -> Self {
        Self {
            editor: "blnvim".into(),
            visual: "blnvim".into(),
            pager: "less".into(),
            less_opts: "-R -i -w -M -z-4 -x4".into(),
            bat_theme: "ansi".into(),
            manpager: "sh -c 'col -bx | bat -l man -p'".into(),
            manroffopt: "-c".into(),
            funcnest: 1000,
            keytimeout: 20,
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            file: "${XDG_STATE_HOME:-$HOME/.local/state}/zsh/history".into(),
            size: 1_000_000,
            save_size: 1_000_000,
            extended: true,
            expire_dups_first: true,
            ignore_all_dups: true,
            find_no_dups: true,
            ignore_space: true,
            verify: true,
            share: true,
            inc_append: true,
            reduce_blanks: true,
        }
    }
}

impl Default for ZshOptions {
    fn default() -> Self {
        Self {
            auto_cd: true,
            auto_pushd: true,
            pushd_ignore_dups: true,
            pushd_minus: true,
            always_to_end: true,
            auto_menu: true,
            auto_list: true,
            complete_in_word: true,
            menu_complete: false,
            extended_glob: true,
            glob_dots: true,
            no_case_glob: true,
            numeric_glob_sort: true,
            no_nomatch: true,
            interactive_comments: true,
            rc_quotes: true,
            combining_chars: true,
            long_list_jobs: true,
            auto_resume: true,
            notify: true,
            bg_nice: false,
            hup: false,
            check_jobs: false,
            beep: false,
            flow_control: false,
        }
    }
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            eza: EzaConfig::default(),
            skim: SkimConfig::default(),
        }
    }
}

impl Default for EzaConfig {
    fn default() -> Self {
        Self {
            icons: true,
            group_directories_first: true,
            ls_wrapper: true,
        }
    }
}

impl Default for SkimConfig {
    fn default() -> Self {
        Self {
            algo: "arinae".into(),
            height: "30%".into(),
            layout: "reverse".into(),
            border: "rounded".into(),
            info: "inline".into(),
            prompt: "❄ ".into(),
            preview_window: "right:50%:hidden:wrap".into(),
            colors: SkimColors::default(),
            bindings: vec![
                "ctrl-/:toggle-preview".into(),
                "ctrl-u:preview-half-page-up".into(),
                "ctrl-d:preview-half-page-down".into(),
            ],
            ctrl_t: SkimWidgetConfig {
                enabled: true,
                scheme: "path".into(),
                command: "fd --type f --type d --hidden --follow --exclude .git --strip-cwd-prefix"
                    .into(),
                preview: "if [ -d {} ]; then eza --tree --level=2 --icons --color=always {} 2>/dev/null; else bat --color=always --style=numbers --line-range=:500 {} 2>/dev/null; fi".into(),
                header: "CTRL-T: Files/Dirs | CTRL-/: Toggle Preview".into(),
            },
            alt_c: SkimWidgetConfig {
                enabled: true,
                scheme: "path".into(),
                command: "fd --type d --hidden --follow --exclude .git --strip-cwd-prefix".into(),
                preview: "eza --tree --level=2 --icons --color=always {}".into(),
                header: "ALT-C: Change Directory".into(),
            },
            ctrl_f: true,
        }
    }
}

impl Default for SkimColors {
    fn default() -> Self {
        Self {
            fg: "#D8DEE9".into(),
            bg: "#2E3440".into(),
            hl: "#88C0D0".into(),
            fg_plus: "#ECEFF4".into(),
            bg_plus: "#3B4252".into(),
            hl_plus: "#8FBCBB".into(),
            info: "#81A1C1".into(),
            prompt: "#A3BE8C".into(),
            pointer: "#BF616A".into(),
            marker: "#B48EAD".into(),
            spinner: "#81A1C1".into(),
            header: "#5E81AC".into(),
            border: "#4C566A".into(),
            query: "#ECEFF4".into(),
        }
    }
}

impl Default for SkimWidgetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scheme: String::new(),
            command: String::new(),
            preview: String::new(),
            header: String::new(),
        }
    }
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            case_insensitive: true,
            partial_word: true,
            substring: true,
            menu_select: true,
            use_cache: true,
            cache_path: "${XDG_CACHE_HOME:-$HOME/.cache}/zsh".into(),
            rehash: true,
            special_dirs: true,
            colors: CompletionColors::default(),
        }
    }
}

impl Default for CompletionColors {
    fn default() -> Self {
        Self {
            corrections: "#A3BE8C".into(),
            descriptions: "#88C0D0".into(),
            messages: "#B48EAD".into(),
            warnings: "#BF616A".into(),
        }
    }
}

impl Default for VimModeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cursor_shape: true,
            system_clipboard: true,
            edit_command_line: true,
        }
    }
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            zoxide: ZoxideConfig::default(),
            atuin: AtuinConfig::default(),
            direnv: DirenvConfig::default(),
            starship: StarshipConfig::default(),
            fzf_tab: FzfTabConfig::default(),
            syntax_highlighting: SyntaxHighlightingConfig::default(),
        }
    }
}

impl Default for ZoxideConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cmd: "cd".into(),
        }
    }
}

impl Default for AtuinConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            disable_up_arrow: true,
            disable_ctrl_r: true,
        }
    }
}

impl Default for DirenvConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_format: String::new(),
            warn_timeout: "10s".into(),
        }
    }
}

impl Default for StarshipConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            deferred: true,
        }
    }
}

impl Default for FzfTabConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fzf_command: "skim-tab".into(),
            use_fzf_default_opts: true,
            switch_group: ["<".into(), ">".into()],
            no_sort: true,
            path_scheme_commands: vec!["cd".into(), "pushd".into(), "z".into()],
            previews: default_fzf_tab_previews(),
        }
    }
}

impl Default for SyntaxHighlightingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            deferred: true,
        }
    }
}

fn default_aliases() -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    // Editor
    m.insert("vim".into(), "blnvim".into());
    m.insert("vi".into(), "blnvim".into());
    m.insert("nvim".into(), "blnvim".into());
    m.insert("vimdiff".into(), "blnvim -d".into());
    // Rust tools
    m.insert("cat".into(), "bat --style=plain --paging=never".into());
    m.insert("catt".into(), "bat".into());
    m.insert("top".into(), "btm".into());
    m.insert("htop".into(), "btm".into());
    m.insert("loc".into(), "tokei".into());
    m.insert("http".into(), "xh".into());
    m.insert("https".into(), "xh --https".into());
    m.insert("hex".into(), "hexyl".into());
    m.insert("sysinfo".into(), "macchina".into());
    m.insert("neofetch".into(), "macchina".into());
    m.insert("mcat".into(), "mdcat".into());
    m.insert("fm".into(), "yazi".into());
    m.insert("lg".into(), "lazygit".into());
    m.insert("gg".into(), "lazygit".into());
    // Navigation
    m.insert("..".into(), "cd ..".into());
    m.insert("...".into(), "cd ../..".into());
    m.insert("....".into(), "cd ../../..".into());
    m.insert(".....".into(), "cd ../../../..".into());
    // Git
    m.insert("g".into(), "git".into());
    m.insert("gs".into(), "git status".into());
    m.insert("ga".into(), "git add".into());
    m.insert("gaa".into(), "git add --all".into());
    m.insert("gc".into(), "git commit".into());
    m.insert("gcm".into(), "git commit -m".into());
    m.insert("gp".into(), "git push".into());
    m.insert("gpl".into(), "git pull".into());
    m.insert("gd".into(), "git diff".into());
    m.insert("gds".into(), "git diff --staged".into());
    m.insert("gl".into(), "git log --oneline --graph --decorate".into());
    m.insert(
        "gla".into(),
        "git log --oneline --graph --decorate --all".into(),
    );
    m.insert("gco".into(), "git checkout".into());
    m.insert("gcb".into(), "git checkout -b".into());
    m.insert("gb".into(), "git branch".into());
    m.insert("gba".into(), "git branch -a".into());
    m.insert("gbd".into(), "git branch -d".into());
    m.insert("grb".into(), "git rebase".into());
    m.insert("grbi".into(), "git rebase -i".into());
    m.insert("gf".into(), "git fetch".into());
    m.insert("gfa".into(), "git fetch --all --prune".into());
    m.insert("gpf".into(), "git push --force-with-lease".into());
    m.insert("gsw".into(), "git switch".into());
    m.insert("gswc".into(), "git switch -c".into());
    m.insert("gr".into(), "git restore".into());
    m.insert("grs".into(), "git restore --staged".into());
    m.insert("gst".into(), "git stash".into());
    m.insert("gstp".into(), "git stash pop".into());
    m.insert("gstl".into(), "git stash list".into());
    // Nix
    m.insert("nix".into(), "noglob nix".into());
    m.insert("nix-shell".into(), "noglob nix-shell --run zsh".into());
    m.insert("nb".into(), "noglob nix build".into());
    m.insert("nd".into(), "noglob nix develop".into());
    m.insert("ns".into(), "noglob nix search nixpkgs".into());
    m.insert("nfu".into(), "noglob nix flake update".into());
    m.insert("ngc".into(), "noglob nix-collect-garbage -d".into());
    // Docker
    m.insert("d".into(), "docker".into());
    m.insert("dc".into(), "docker compose".into());
    m.insert("dps".into(), "docker ps".into());
    m.insert("dpsa".into(), "docker ps -a".into());
    m.insert("di".into(), "docker images".into());
    m.insert("dex".into(), "docker exec -it".into());
    m.insert("dl".into(), "docker logs".into());
    m.insert("dlf".into(), "docker logs -f".into());
    m.insert("dcu".into(), "docker compose up".into());
    m.insert("dcud".into(), "docker compose up -d".into());
    m.insert("dcd".into(), "docker compose down".into());
    // Kubernetes
    m.insert("kgd".into(), "kubectl get deployments".into());
    m.insert("kd".into(), "kubectl describe".into());
    m.insert("kdp".into(), "kubectl describe pod".into());
    m.insert("kl".into(), "kubectl logs".into());
    m.insert("klf".into(), "kubectl logs -f".into());
    // File ops
    m.insert("cp".into(), "cp -i".into());
    m.insert("mv".into(), "mv -i".into());
    m.insert("rm".into(), "rm -i".into());
    m.insert("mkdir".into(), "mkdir -p".into());
    m.insert("df".into(), "df -h".into());
    // Util
    m.insert("reload".into(), "source ${ZDOTDIR:-$HOME}/.zshrc".into());
    m.insert("path".into(), "echo ${(F)path}".into());
    m.insert("py".into(), "python3".into());
    m.insert("python".into(), "python3".into());
    m.insert("please".into(), "sudo".into());
    m.insert("pls".into(), "sudo".into());
    // Cargo
    m.insert("cr".into(), "cargo run".into());
    m.insert("ct".into(), "cargo test".into());
    m.insert("cb".into(), "cargo build".into());
    m.insert("cbr".into(), "cargo build --release".into());
    m.insert("cc".into(), "cargo check".into());
    m.insert("cl".into(), "cargo clippy".into());
    m.insert("cf".into(), "cargo fmt".into());
    m.insert("cu".into(), "cargo update".into());
    m
}

fn default_fzf_tab_previews() -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    m.insert(
        "cd".into(),
        "eza --tree --level=2 --icons --color=always $realpath 2>/dev/null".into(),
    );
    m.insert(
        "pushd".into(),
        "eza --tree --level=2 --icons --color=always $realpath 2>/dev/null".into(),
    );
    m.insert(
        "z".into(),
        "eza --tree --level=2 --icons --color=always $realpath 2>/dev/null".into(),
    );
    m.insert("*".into(), "if [[ -d $realpath ]]; then eza --tree --level=2 --icons --color=always $realpath 2>/dev/null; elif [[ -f $realpath ]]; then bat --color=always --style=numbers --line-range=:200 $realpath 2>/dev/null; fi".into());
    m
}

/// Load config from XDG path with BLX_CONFIG env override.
///
/// Returns defaults if no config file exists. Provides contextual error
/// messages on parse failure (file path, hint to dump-config).
pub fn load_config() -> anyhow::Result<ShellConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(ShellConfig::default());
    }
    let contents = std::fs::read_to_string(&path).map_err(|e| {
        anyhow::anyhow!("cannot read config file {}: {e}", path.display())
    })?;
    serde_yaml::from_str::<ShellConfig>(&contents).map_err(|e| {
        anyhow::anyhow!(
            "config parse error in {}:\n  {e}\n\n\
             hint: run 'blx init dump-config' to see the expected format",
            path.display()
        )
    })
}

/// Determine config file path: BLX_CONFIG env > ~/.config/blx/blx.yaml
///
/// Uses `$HOME/.config` (XDG convention) on all platforms — not
/// `dirs::config_dir()` which returns `~/Library/Application Support` on macOS.
pub fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("BLX_CONFIG") {
        return PathBuf::from(p);
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join(".config").join("blx").join("blx.yaml")
}

/// Parse a YAML string into a `ShellConfig`.
///
/// Missing fields fall back to `#[serde(default)]` values.
/// Used by tests and available for external callers that have YAML in memory.
#[allow(dead_code)]
pub fn parse_config(yaml: &str) -> anyhow::Result<ShellConfig> {
    Ok(serde_yaml::from_str(yaml)?)
}

/// Write the default config to a path (for `blx init --dump-config`).
pub fn dump_default_config() -> anyhow::Result<String> {
    let config = ShellConfig::default();
    Ok(serde_yaml::to_string(&config)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrips_through_yaml() {
        let original = ShellConfig::default();
        let yaml = serde_yaml::to_string(&original).unwrap();
        let parsed: ShellConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn empty_yaml_yields_defaults() {
        let config: ShellConfig = serde_yaml::from_str("").unwrap();
        assert_eq!(config, ShellConfig::default());
    }

    #[test]
    fn partial_yaml_merges_with_defaults() {
        let yaml = "shell:\n  editor: vim\n  keytimeout: 10\n";
        let config = parse_config(yaml).unwrap();
        assert_eq!(config.shell.editor, "vim");
        assert_eq!(config.shell.keytimeout, 10);
        // Unset fields use defaults
        assert_eq!(config.shell.visual, "blnvim");
        assert_eq!(config.shell.pager, "less");
        assert_eq!(config.history.size, 1_000_000);
    }

    #[test]
    fn partial_yaml_disabling_plugins() {
        let yaml = "plugins:\n  starship:\n    enabled: false\n  atuin:\n    enabled: false\n";
        let config = parse_config(yaml).unwrap();
        assert!(!config.plugins.starship.enabled);
        assert!(!config.plugins.atuin.enabled);
        // Others still enabled
        assert!(config.plugins.zoxide.enabled);
        assert!(config.plugins.direnv.enabled);
        assert!(config.plugins.fzf_tab.enabled);
    }

    #[test]
    fn invalid_yaml_type_returns_error() {
        let yaml = "shell:\n  funcnest: not_a_number\n";
        let err = parse_config(yaml).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("funcnest"), "error should mention the field: {msg}");
        assert!(msg.contains("invalid type"), "error should describe the type mismatch: {msg}");
    }

    #[test]
    fn invalid_yaml_syntax_returns_error() {
        let yaml = "shell:\n  editor: [unclosed\n";
        assert!(parse_config(yaml).is_err());
    }

    #[test]
    fn custom_aliases_override_defaults() {
        let yaml = "aliases:\n  vim: nvim\n  custom: 'echo hello'\n";
        let config = parse_config(yaml).unwrap();
        assert_eq!(config.aliases.get("vim").unwrap(), "nvim");
        assert_eq!(config.aliases.get("custom").unwrap(), "echo hello");
        // Default aliases NOT present — aliases is fully replaced, not merged
        assert!(!config.aliases.contains_key("cat"));
    }

    #[test]
    fn custom_environment_vars() {
        let yaml = "environment:\n  MY_VAR: my_value\n  ANOTHER: '123'\n";
        let config = parse_config(yaml).unwrap();
        assert_eq!(config.environment.get("MY_VAR").unwrap(), "my_value");
        assert_eq!(config.environment.get("ANOTHER").unwrap(), "123");
    }

    #[test]
    fn extra_zsh_lines() {
        let yaml = "extra_zsh:\n  - 'source ~/.local.zsh'\n  - 'export FOO=bar'\n";
        let config = parse_config(yaml).unwrap();
        assert_eq!(config.extra_zsh.len(), 2);
        assert_eq!(config.extra_zsh[0], "source ~/.local.zsh");
    }

    #[test]
    fn fzf_tab_switch_group_array() {
        let yaml = "plugins:\n  fzf_tab:\n    switch_group: ['[', ']']\n";
        let config = parse_config(yaml).unwrap();
        assert_eq!(config.plugins.fzf_tab.switch_group, ["[", "]"]);
    }

    #[test]
    fn dump_default_config_is_valid_yaml() {
        let yaml = dump_default_config().unwrap();
        assert!(!yaml.is_empty());
        // Must parse back cleanly
        let _: ShellConfig = serde_yaml::from_str(&yaml).unwrap();
    }

    #[test]
    fn default_aliases_not_empty() {
        let config = ShellConfig::default();
        assert!(!config.aliases.is_empty());
        assert!(config.aliases.contains_key("gs"));
        assert!(config.aliases.contains_key("cat"));
    }

    #[test]
    fn default_fzf_tab_previews_include_wildcard() {
        let config = ShellConfig::default();
        assert!(config.plugins.fzf_tab.previews.contains_key("*"));
        assert!(config.plugins.fzf_tab.previews.contains_key("cd"));
    }

    #[test]
    fn skim_colors_all_populated() {
        let colors = SkimColors::default();
        assert!(!colors.fg.is_empty());
        assert!(!colors.bg.is_empty());
        assert!(!colors.hl.is_empty());
        assert!(!colors.fg_plus.is_empty());
        assert!(!colors.bg_plus.is_empty());
        assert!(!colors.hl_plus.is_empty());
        assert!(!colors.info.is_empty());
        assert!(!colors.prompt.is_empty());
        assert!(!colors.pointer.is_empty());
        assert!(!colors.marker.is_empty());
        assert!(!colors.spinner.is_empty());
        assert!(!colors.header.is_empty());
        assert!(!colors.border.is_empty());
        assert!(!colors.query.is_empty());
    }
}
