use crate::config::{self, ShellConfig};
use clap::{Args, Subcommand};
use colored::Colorize;
use std::fmt::Write;

/// Escape a string for embedding inside single quotes in zsh.
/// Replaces `'` with `'\''` (end quote, escaped quote, start quote).
fn shell_quote(s: &str) -> String {
    s.replace('\'', "'\\''")
}

#[derive(Args)]
#[command(
    long_about = "Generate shell configuration from ~/.config/blx/blx.yaml.\n\n\
                   Quick start:\n  \
                   1. blx init dump-config > ~/.config/blx/blx.yaml\n  \
                   2. Edit the YAML to taste\n  \
                   3. Add to .zshrc: eval \"$(blx init zsh)\""
)]
pub struct InitArgs {
    #[command(subcommand)]
    command: InitCommand,
}

#[derive(Subcommand)]
pub enum InitCommand {
    /// Generate zsh configuration to stdout (eval "$(blx init zsh)")
    #[command(
        long_about = "Generate zsh configuration to stdout.\n\n\
                       Usage in .zshrc:\n  \
                       eval \"$(blx init zsh)\"\n\n\
                       Reads config from ~/.config/blx/blx.yaml (or BLX_CONFIG env).\n\
                       Falls back to built-in defaults if no config file exists."
    )]
    Zsh {
        /// Validate config and report issues without emitting zsh
        #[arg(long)]
        check: bool,
    },
    /// Dump the default YAML config to stdout
    #[command(
        long_about = "Dump the full default YAML config to stdout.\n\n\
                       Redirect to create your config file:\n  \
                       mkdir -p ~/.config/blx && blx init dump-config > ~/.config/blx/blx.yaml\n\n\
                       Then edit the YAML — only set what you want to override.\n\
                       Omitted fields use built-in defaults."
    )]
    DumpConfig,
    /// Create config file at ~/.config/blx/blx.yaml
    #[command(
        long_about = "Create the config file with defaults if it doesn't exist.\n\n\
                       Use --force to overwrite an existing config.\n\
                       Use --edit to open the file in $EDITOR after creation."
    )]
    Config {
        /// Overwrite existing config file
        #[arg(long)]
        force: bool,
        /// Open config in $EDITOR after creation
        #[arg(long, short)]
        edit: bool,
    },
}

impl InitArgs {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            InitCommand::Zsh { check } => {
                if check {
                    return run_check();
                }
                let config = config::load_config()?;
                let output = generate_zsh(&config);
                print!("{output}");
                Ok(())
            }
            InitCommand::DumpConfig => {
                let yaml = config::dump_default_config()?;
                print!("{yaml}");
                Ok(())
            }
            InitCommand::Config { force, edit } => run_config_bootstrap(force, edit),
        }
    }
}

/// Validate config and report issues to stderr.
fn run_check() -> anyhow::Result<()> {
    let path = config::config_path();
    if !path.exists() {
        eprintln!(
            "{} no config file at {} — using built-in defaults",
            "info:".cyan().bold(),
            path.display()
        );
        eprintln!("{}", "ok".green().bold());
        return Ok(());
    }

    let contents = std::fs::read_to_string(&path).map_err(|e| {
        anyhow::anyhow!("cannot read {}: {e}", path.display())
    })?;

    match serde_yaml_ng::from_str::<ShellConfig>(&contents) {
        Ok(config) => {
            let warnings = validate_config(&config);
            eprintln!(
                "{} {} parsed successfully",
                "ok:".green().bold(),
                path.display()
            );
            for w in &warnings {
                eprintln!("  {} {w}", "warn:".yellow().bold());
            }
            if warnings.is_empty() {
                eprintln!("  no issues found");
            }
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "{} config parse error in {}:\n  {e}\n",
                "error:".red().bold(),
                path.display()
            );
            eprintln!(
                "  {} run 'blx init dump-config' to see the expected format",
                "hint:".cyan().bold()
            );
            anyhow::bail!("config validation failed");
        }
    }
}

/// Check config for potential issues (not syntax errors, but semantic warnings).
fn validate_config(config: &ShellConfig) -> Vec<String> {
    let mut warnings = Vec::new();

    if config.shell.editor.is_empty() {
        warnings.push("shell.editor is empty — $EDITOR will be unset".into());
    }
    if config.history.size == 0 {
        warnings.push("history.size is 0 — no history will be kept".into());
    }
    if config.history.save_size == 0 {
        warnings.push("history.save_size is 0 — history will not persist".into());
    }
    if config.plugins.fzf_tab.enabled && !config.plugins.fzf_tab.use_fzf_default_opts {
        warnings.push(
            "plugins.fzf_tab is enabled but use_fzf_default_opts is false — \
             skim colors won't apply to tab completion"
                .into(),
        );
    }
    if config.vim_mode.enabled && config.shell.keytimeout > 100 {
        warnings.push(format!(
            "shell.keytimeout is {} — high values cause sluggish vim mode switching",
            config.shell.keytimeout
        ));
    }

    warnings
}

/// Bootstrap the config file.
fn run_config_bootstrap(force: bool, edit: bool) -> anyhow::Result<()> {
    let path = config::config_path();
    let existed = path.exists();

    if existed && !force {
        eprintln!(
            "{} config already exists at {}",
            "skip:".yellow().bold(),
            path.display()
        );
        eprintln!(
            "  use {} to overwrite, or edit directly",
            "--force".bold()
        );
        return Ok(());
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let yaml = config::dump_default_config()?;
    std::fs::write(&path, &yaml)?;

    let verb = if existed { "overwrote" } else { "created" };
    eprintln!(
        "{} {verb} {}",
        "done:".green().bold(),
        path.display()
    );
    eprintln!(
        "  add to .zshrc: {}",
        "eval \"$(blx init zsh)\"".bold()
    );

    if edit {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".into());
        let status = std::process::Command::new(&editor)
            .arg(&path)
            .status()?;
        if !status.success() {
            anyhow::bail!("{editor} exited with {status}");
        }
    }

    Ok(())
}

/// Generate complete zsh configuration from a `ShellConfig`.
fn generate_zsh(config: &ShellConfig) -> String {
    let mut out = String::with_capacity(8192);

    out.push_str("# Generated by blx init zsh — do not edit\n\n");

    emit_options(&mut out, config);
    emit_history(&mut out, config);
    emit_environment(&mut out, config);
    emit_completion(&mut out, config);
    emit_vim_mode(&mut out, config);
    emit_aliases(&mut out, config);
    emit_eza_wrapper(&mut out, config);
    emit_skim(&mut out, config);
    emit_plugins(&mut out, config);
    emit_extra(&mut out, config);

    out
}

// ========== Options ==========

fn emit_options(out: &mut String, config: &ShellConfig) {
    out.push_str("# ===== Shell Options =====\n");
    let opts = &config.options;

    macro_rules! opt {
        ($field:expr, $name:expr) => {
            if $field {
                let _ = writeln!(out, "setopt {}", $name);
            } else {
                let _ = writeln!(out, "unsetopt {}", $name);
            }
        };
    }

    // Directory
    opt!(opts.auto_cd, "AUTO_CD");
    opt!(opts.auto_pushd, "AUTO_PUSHD");
    opt!(opts.pushd_ignore_dups, "PUSHD_IGNORE_DUPS");
    opt!(opts.pushd_minus, "PUSHD_MINUS");

    // Completion
    opt!(opts.always_to_end, "ALWAYS_TO_END");
    opt!(opts.auto_menu, "AUTO_MENU");
    opt!(opts.auto_list, "AUTO_LIST");
    opt!(opts.complete_in_word, "COMPLETE_IN_WORD");
    opt!(opts.menu_complete, "MENU_COMPLETE");

    // Globbing
    opt!(opts.extended_glob, "EXTENDED_GLOB");
    opt!(opts.glob_dots, "GLOB_DOTS");
    opt!(opts.no_case_glob, "NO_CASE_GLOB");
    opt!(opts.numeric_glob_sort, "NUMERIC_GLOB_SORT");
    opt!(opts.no_nomatch, "NO_NOMATCH");

    // I/O
    opt!(opts.interactive_comments, "INTERACTIVE_COMMENTS");
    opt!(opts.rc_quotes, "RC_QUOTES");
    opt!(opts.combining_chars, "COMBINING_CHARS");

    // Jobs
    opt!(opts.long_list_jobs, "LONG_LIST_JOBS");
    opt!(opts.auto_resume, "AUTO_RESUME");
    opt!(opts.notify, "NOTIFY");
    opt!(opts.bg_nice, "BG_NICE");
    opt!(opts.hup, "HUP");
    opt!(opts.check_jobs, "CHECK_JOBS");

    // Performance
    opt!(opts.beep, "BEEP");
    opt!(opts.flow_control, "FLOW_CONTROL");

    out.push('\n');
}

// ========== History ==========

fn emit_history(out: &mut String, config: &ShellConfig) {
    let h = &config.history;
    out.push_str("# ===== History =====\n");
    let _ = writeln!(out, "export HISTFILE=\"{}\"", h.file);
    let _ = writeln!(
        out,
        "[[ -d \"${{HISTFILE:h}}\" ]] || mkdir -p \"${{HISTFILE:h}}\""
    );
    let _ = writeln!(out, "export HISTSIZE={}", h.size);
    let _ = writeln!(out, "export SAVEHIST={}", h.save_size);

    macro_rules! hist_opt {
        ($field:expr, $name:expr) => {
            if $field {
                let _ = writeln!(out, "setopt {}", $name);
            }
        };
    }

    hist_opt!(h.extended, "EXTENDED_HISTORY");
    hist_opt!(h.expire_dups_first, "HIST_EXPIRE_DUPS_FIRST");
    hist_opt!(h.ignore_all_dups, "HIST_IGNORE_ALL_DUPS");
    hist_opt!(h.find_no_dups, "HIST_FIND_NO_DUPS");
    hist_opt!(h.ignore_space, "HIST_IGNORE_SPACE");
    hist_opt!(h.verify, "HIST_VERIFY");
    hist_opt!(h.share, "SHARE_HISTORY");
    hist_opt!(h.inc_append, "INC_APPEND_HISTORY");
    hist_opt!(h.reduce_blanks, "HIST_REDUCE_BLANKS");

    out.push('\n');
}

// ========== Environment ==========

fn emit_environment(out: &mut String, config: &ShellConfig) {
    let s = &config.shell;
    out.push_str("# ===== Environment =====\n");
    let _ = writeln!(out, "export EDITOR='{}'", shell_quote(&s.editor));
    let _ = writeln!(out, "export VISUAL='{}'", shell_quote(&s.visual));
    let _ = writeln!(out, "export PAGER='{}'", shell_quote(&s.pager));
    let _ = writeln!(out, "export LESS='{}'", shell_quote(&s.less_opts));
    out.push_str("export LESSHISTFILE='-'\n");
    let _ = writeln!(out, "export BAT_THEME='{}'", shell_quote(&s.bat_theme));
    let _ = writeln!(out, "export MANPAGER='{}'", shell_quote(&s.manpager));
    let _ = writeln!(out, "export MANROFFOPT='{}'", shell_quote(&s.manroffopt));
    let _ = writeln!(out, "export FUNCNEST={}", s.funcnest);
    let _ = writeln!(out, "export KEYTIMEOUT={}", s.keytimeout);
    out.push_str("export CLICOLOR=1\n");
    out.push_str("export LSCOLORS=\"ExGxBxDxCxEgEdxbxgxcxd\"\n");

    // Extra user-defined environment variables
    for (k, v) in &config.environment {
        let _ = writeln!(out, "export {k}='{}'", shell_quote(v));
    }

    out.push('\n');
}

// ========== Completion System ==========

fn emit_completion(out: &mut String, config: &ShellConfig) {
    let c = &config.completion;
    out.push_str("# ===== Completion System =====\n");

    // Cache directory + compinit
    let _ = writeln!(
        out,
        "local _zsh_cache=\"{}\"",
        c.cache_path
    );
    out.push_str("[[ -d \"$_zsh_cache\" ]] || mkdir -p \"$_zsh_cache\"\n");
    out.push_str("autoload -Uz compinit\n");
    out.push_str("local zcompdump=\"$_zsh_cache/zcompdump-$HOST\"\n");
    out.push_str(
        "if [[ -n ${zcompdump}(#qN.mh+24) ]] || [[ ! -f \"$zcompdump\" ]]; then\n  compinit -d \"$zcompdump\"\nelse\n  compinit -C -d \"$zcompdump\"\nfi\n",
    );
    out.push_str(
        "{ [[ ! ${zcompdump}.zwc -nt ${zcompdump} ]] && zcompile \"${zcompdump}\" } &!\n",
    );

    // Caching
    if c.use_cache {
        out.push_str("zstyle ':completion::complete:*' use-cache on\n");
        let _ = writeln!(
            out,
            "zstyle ':completion::complete:*' cache-path \"$_zsh_cache\""
        );
    }

    // Matching
    let mut matcher_parts: Vec<&str> = Vec::new();
    if c.case_insensitive {
        matcher_parts.push("'m:{a-zA-Z}={A-Za-z}'");
    }
    if c.partial_word {
        matcher_parts.push("'r:|[._-]=* r:|=*'");
    }
    if c.substring {
        matcher_parts.push("'l:|=* r:|=*'");
    }
    if !matcher_parts.is_empty() {
        let _ = writeln!(
            out,
            "zstyle ':completion:*' matcher-list {}",
            matcher_parts.join(" ")
        );
    }

    // Menu
    if c.menu_select {
        out.push_str("zstyle ':completion:*:*:*:*:*' menu select\n");
    }

    // Groups and formatting
    out.push_str("zstyle ':completion:*:matches' group 'yes'\n");
    out.push_str("zstyle ':completion:*:options' description 'yes'\n");
    out.push_str("zstyle ':completion:*:options' auto-description '%d'\n");
    let _ = writeln!(
        out,
        "zstyle ':completion:*:corrections' format ' %F{{{}}}-- %d (errors: %e) --%f'",
        c.colors.corrections
    );
    let _ = writeln!(
        out,
        "zstyle ':completion:*:descriptions' format ' %F{{{}}}-- %d --%f'",
        c.colors.descriptions
    );
    let _ = writeln!(
        out,
        "zstyle ':completion:*:messages' format ' %F{{{}}}-- %d --%f'",
        c.colors.messages
    );
    let _ = writeln!(
        out,
        "zstyle ':completion:*:warnings' format ' %F{{{}}}-- no matches found --%f'",
        c.colors.warnings
    );
    let _ = writeln!(
        out,
        "zstyle ':completion:*:default' list-prompt '%S%M matches%s'"
    );
    let _ = writeln!(
        out,
        "zstyle ':completion:*' format ' %F{{{}}}-- %d --%f'",
        c.colors.descriptions
    );
    out.push_str("zstyle ':completion:*' group-name ''\n");
    out.push_str("zstyle ':completion:*' verbose yes\n");

    // Colors from LS_COLORS
    out.push_str("zstyle ':completion:*' list-colors \"${(s.:.)LS_COLORS}\"\n");
    out.push_str("zstyle ':completion:*:*:kill:*:processes' list-colors '=(#b) #([0-9]#) ([0-9a-z-]#)*=01;34=0=01'\n");

    // Fuzzy matching
    out.push_str("zstyle -e ':completion:*:approximate:*' max-errors 'reply=($((($#PREFIX+$#SUFFIX)/3>7?7:($#PREFIX+$#SUFFIX)/3))numeric)'\n");

    // Users filter
    out.push_str(concat!(
        "zstyle ':completion:*:*:*:users' ignored-patterns \\\n",
        "  adm amanda apache at avahi avahi-autoipd beaglidx bin cacti canna \\\n",
        "  clamav daemon dbus distcache dnsmasq dovecot fax ftp games gdm \\\n",
        "  gkrellmd gopher hacluster haldaemon halt hsqldb ident junkbust kdm \\\n",
        "  ldap lp mail mailman mailnull man messagebus mldonkey mysql nagios \\\n",
        "  named netdump news nfsnobody nobody nscd ntp nut nx obsrun openvpn \\\n",
        "  operator pcap polkitd postfix postgres privoxy pulse pvm quagga radvd \\\n",
        "  rpc rpcuser rpm rtkit scard shutdown squid sshd statd svn sync tftp \\\n",
        "  usbmux uucp vcsa wwwrun xfs '_*'\n",
    ));
    out.push_str("zstyle '*' single-ignored show\n");

    // Process completion
    out.push_str("zstyle ':completion:*:*:*:*:processes' command \"ps -u $USER -o pid,user,comm -w -w\"\n");
    out.push_str("zstyle ':completion:*:*:kill:*' menu yes select\n");
    out.push_str("zstyle ':completion:*:*:kill:*' force-list always\n");
    out.push_str("zstyle ':completion:*:*:kill:*' insert-ids single\n");

    // Hostname
    out.push_str("zstyle ':completion:*:ssh:*' hosts off\n");
    out.push_str("zstyle ':completion:*:scp:*' hosts off\n");

    // Functions
    out.push_str("zstyle ':completion:*:functions' ignored-patterns '_*'\n");
    out.push_str("zstyle ':completion:*:*:-subscript-:*' tag-order indexes parameters\n");

    // cd
    out.push_str("zstyle ':completion:*:cd:*' ignore-parents parent pwd\n");
    if c.special_dirs {
        out.push_str("zstyle ':completion:*' special-dirs true\n");
    }

    // Performance
    out.push_str("zstyle ':completion:*' accept-exact '*(N)'\n");
    out.push_str("zstyle ':completion:*' accept-exact-dirs true\n");
    out.push_str("zstyle ':completion:*:manuals' separate-sections true\n");
    out.push_str("zstyle ':completion:*:manuals.*' insert-sections true\n");

    if c.rehash {
        out.push_str("zstyle ':completion:*' rehash true\n");
    }

    out.push('\n');
}

// ========== Vim Mode ==========

fn emit_vim_mode(out: &mut String, config: &ShellConfig) {
    let v = &config.vim_mode;
    if !v.enabled {
        out.push_str("# Vim mode disabled\nbindkey -e\n\n");
        return;
    }

    out.push_str("# ===== Vim Mode =====\nbindkey -v\n");

    if v.cursor_shape {
        out.push_str(concat!(
            "function zle-keymap-select {\n",
            "  if [[ ${KEYMAP} == vicmd ]] || [[ $1 = 'block' ]]; then\n",
            "    echo -ne '\\e[1 q'\n",
            "  elif [[ ${KEYMAP} == main ]] || [[ ${KEYMAP} == viins ]] || [[ ${KEYMAP} = '' ]] || [[ $1 = 'beam' ]]; then\n",
            "    echo -ne '\\e[5 q'\n",
            "  fi\n",
            "}\n",
            "zle -N zle-keymap-select\n",
            "echo -ne '\\e[5 q'\n",
            "_blx_preexec() { echo -ne '\\e[5 q' }\n",
            "autoload -Uz add-zsh-hook\n",
            "add-zsh-hook preexec _blx_preexec\n",
        ));
    }

    // Key bindings
    out.push_str(concat!(
        "bindkey -M vicmd 'k' up-line-or-history\n",
        "bindkey -M vicmd 'j' down-line-or-history\n",
        "bindkey -M vicmd '^A' beginning-of-line\n",
        "bindkey -M vicmd '^E' end-of-line\n",
        "bindkey -M viins '^A' beginning-of-line\n",
        "bindkey -M viins '^E' end-of-line\n",
        "bindkey -M vicmd '/' history-incremental-search-backward\n",
        "bindkey -M vicmd '?' history-incremental-search-forward\n",
    ));

    if v.edit_command_line {
        out.push_str(concat!(
            "autoload -Uz edit-command-line\n",
            "zle -N edit-command-line\n",
            "bindkey -M vicmd 'v' edit-command-line\n",
        ));
    }

    // Backspace, delete, word movement, history prefix search
    out.push_str(concat!(
        "bindkey -M viins '^?' backward-delete-char\n",
        "bindkey -M viins '^H' backward-delete-char\n",
        "bindkey -M viins '^[[3~' delete-char\n",
        "bindkey -M vicmd '^[[3~' delete-char\n",
        "bindkey -M viins '^[[1;5C' forward-word\n",
        "bindkey -M viins '^[[1;5D' backward-word\n",
        "bindkey -M viins '^[[1;3C' forward-word\n",
        "bindkey -M viins '^[[1;3D' backward-word\n",
        "autoload -Uz up-line-or-beginning-search down-line-or-beginning-search\n",
        "zle -N up-line-or-beginning-search\n",
        "zle -N down-line-or-beginning-search\n",
        "bindkey -M viins '^[[A' up-line-or-beginning-search\n",
        "bindkey -M viins '^[[B' down-line-or-beginning-search\n",
        "bindkey -M vicmd '^[[A' up-line-or-beginning-search\n",
        "bindkey -M vicmd '^[[B' down-line-or-beginning-search\n",
        "bindkey -M viins '^P' up-line-or-beginning-search\n",
        "bindkey -M viins '^N' down-line-or-beginning-search\n",
    ));

    if v.system_clipboard {
        out.push_str(concat!(
            "function vi-yank-clip {\n",
            "  zle vi-yank\n",
            "  if [[ \"$(uname)\" == \"Darwin\" ]]; then\n",
            "    echo \"$CUTBUFFER\" | pbcopy\n",
            "  elif [[ -n \"$WAYLAND_DISPLAY\" ]]; then\n",
            "    echo \"$CUTBUFFER\" | wl-copy\n",
            "  elif [[ -n \"$DISPLAY\" ]]; then\n",
            "    echo \"$CUTBUFFER\" | xsel --clipboard\n",
            "  fi\n",
            "}\n",
            "zle -N vi-yank-clip\n",
            "bindkey -M vicmd 'y' vi-yank-clip\n",
        ));
    }

    out.push('\n');
}

// ========== Aliases ==========

fn emit_aliases(out: &mut String, config: &ShellConfig) {
    out.push_str("# ===== Aliases =====\n");

    for (name, value) in &config.aliases {
        let _ = writeln!(out, "alias {name}='{}'", shell_quote(value));
    }

    // Platform-specific aliases (always emitted, independent of alias map)
    out.push_str(concat!(
        "if [[ \"$(uname)\" == \"Linux\" ]]; then\n",
        "  if [[ -n \"$WAYLAND_DISPLAY\" ]]; then\n",
        "    alias pbcopy='wl-copy'\n",
        "    alias pbpaste='wl-paste'\n",
        "  elif [[ -n \"$DISPLAY\" ]]; then\n",
        "    alias pbcopy='xsel --clipboard --input'\n",
        "    alias pbpaste='xsel --clipboard --output'\n",
        "  fi\n",
        "fi\n",
        "if [[ \"$(uname)\" != \"Darwin\" ]]; then\n",
        "  alias chown='chown --preserve-root'\n",
        "  alias chmod='chmod --preserve-root'\n",
        "  alias chgrp='chgrp --preserve-root'\n",
        "fi\n",
        "if [[ \"$(uname)\" == \"Darwin\" ]]; then\n",
        "  alias ports='lsof -i -n -P | grep LISTEN'\n",
        "  alias nrb='noglob darwin-rebuild switch --flake .'\n",
        "else\n",
        "  alias ports='ss -tulanp'\n",
        "  alias nrb='noglob sudo nixos-rebuild switch --flake .'\n",
        "fi\n",
        "if command -v systemctl &>/dev/null; then\n",
        "  alias sc='sudo systemctl'\n",
        "  alias scs='sudo systemctl status'\n",
        "  alias scr='sudo systemctl restart'\n",
        "  alias sce='sudo systemctl enable'\n",
        "  alias scd='sudo systemctl disable'\n",
        "  alias scst='sudo systemctl start'\n",
        "  alias scsp='sudo systemctl stop'\n",
        "fi\n",
    ));

    out.push('\n');
}

// ========== Eza ls() wrapper ==========

fn emit_eza_wrapper(out: &mut String, config: &ShellConfig) {
    let eza = &config.tools.eza;
    if !eza.ls_wrapper {
        return;
    }

    out.push_str("# ===== ls() → eza wrapper =====\n");

    let mut base_flags = Vec::new();
    if eza.icons {
        base_flags.push("--icons");
    }
    if eza.group_directories_first {
        base_flags.push("--group-directories-first");
    }
    let base = base_flags.join(" ");

    let _ = writeln!(out, concat!(
        "ls() {{\n",
        "  local -a eza_args=({base})\n",
        "  local -a paths=()\n",
        "  local has_l=0 has_a=0 has_t=0 has_r=0 has_1=0\n",
        "  for arg in \"$@\"; do\n",
        "    if [[ \"$arg\" == --* ]]; then\n",
        "      eza_args+=(\"$arg\")\n",
        "    elif [[ \"$arg\" == -* ]]; then\n",
        "      [[ \"$arg\" == *l* ]] && has_l=1\n",
        "      [[ \"$arg\" == *[aA]* ]] && has_a=1\n",
        "      [[ \"$arg\" == *t* ]] && has_t=1\n",
        "      [[ \"$arg\" == *r* ]] && has_r=1\n",
        "      [[ \"$arg\" == *1* ]] && has_1=1\n",
        "    else\n",
        "      paths+=(\"$arg\")\n",
        "    fi\n",
        "  done\n",
        "  (( has_l )) && eza_args+=(\"-l\")\n",
        "  (( has_a )) && eza_args+=(\"-a\")\n",
        "  (( has_t )) && eza_args+=(--sort=modified)\n",
        "  (( has_r )) && eza_args+=(--reverse)\n",
        "  (( has_1 )) && eza_args+=(\"-1\")\n",
        "  eza \"${{eza_args[@]}}\" \"${{paths[@]}}\"\n",
        "}}"),
        base = base
    );

    // Eza aliases
    let _ = writeln!(out, "alias l='eza {base}'");
    let _ = writeln!(out, "alias ll='eza -l {base}'");
    let _ = writeln!(out, "alias la='eza -la {base}'");
    let _ = writeln!(out, "alias lt='eza -T {base}'");
    let _ = writeln!(out, "alias lta='eza -la --sort=modified --reverse {base}'");
    let _ = writeln!(out, "alias ltr='eza -l --sort=modified --reverse {base}'");
    let _ = writeln!(out, "alias tree='eza --tree {base}'");

    out.push('\n');
}

// ========== Skim ==========

fn emit_skim(out: &mut String, config: &ShellConfig) {
    let sk = &config.tools.skim;
    out.push_str("# ===== Skim (fuzzy finder) =====\n");

    // Build SKIM_DEFAULT_OPTIONS
    let colors = &sk.colors;
    let color_str = format!(
        "fg:{},bg:{},hl:{},fg+:{},bg+:{},hl+:{},info:{},prompt:{},pointer:{},marker:{},spinner:{},header:{},border:{},query:{}",
        colors.fg, colors.bg, colors.hl, colors.fg_plus, colors.bg_plus, colors.hl_plus,
        colors.info, colors.prompt, colors.pointer, colors.marker, colors.spinner,
        colors.header, colors.border, colors.query
    );

    let mut parts = Vec::new();
    parts.push(format!("--algo={}", sk.algo));
    parts.push(format!("--height {}", sk.height));
    parts.push(format!("--layout={}", sk.layout));
    parts.push(format!("--border={}", sk.border));
    parts.push(format!("--info={}", sk.info));
    parts.push(format!("--prompt='{}'", sk.prompt));
    parts.push("--ansi".into());
    for b in &sk.bindings {
        parts.push(format!("--bind='{b}'"));
    }
    parts.push(format!("--preview-window='{}'", sk.preview_window));
    parts.push(format!("--color={color_str}"));

    let opts = parts.join("\n  ");
    let _ = writeln!(out, "export SKIM_DEFAULT_OPTIONS=\"\n  {opts}\n\"");

    // Bridge for fzf-tab
    out.push_str("export FZF_DEFAULT_OPTS=\"$SKIM_DEFAULT_OPTIONS\"\n");
    out.push_str("export SKIM_DEFAULT_COMMAND='fd --type f --hidden --follow --exclude .git --strip-cwd-prefix'\n");

    // Ctrl+T widget
    let ct = &sk.ctrl_t;
    if ct.enabled {
        let _ = writeln!(out, "export SKIM_CTRL_T_COMMAND='{}'", shell_quote(&ct.command));
        let mut ct_opts = Vec::new();
        if !ct.scheme.is_empty() {
            ct_opts.push(format!("--scheme={}", ct.scheme));
        }
        ct_opts.push(format!("--preview '{}'", ct.preview));
        ct_opts.push("--bind 'ctrl-/:change-preview-window(down|hidden|)'".into());
        ct_opts.push(format!("--header '{}'", ct.header));
        let _ = writeln!(
            out,
            "export SKIM_CTRL_T_OPTS=\"\n  {}\n\"",
            ct_opts.join("\n  ")
        );
    }

    // Alt+C widget
    let ac = &sk.alt_c;
    if ac.enabled {
        let _ = writeln!(out, "export SKIM_ALT_C_COMMAND='{}'", shell_quote(&ac.command));
        let mut ac_opts = Vec::new();
        if !ac.scheme.is_empty() {
            ac_opts.push(format!("--scheme={}", ac.scheme));
        }
        ac_opts.push(format!("--preview '{}'", ac.preview));
        ac_opts.push(format!("--header '{}'", ac.header));
        let _ = writeln!(
            out,
            "export SKIM_ALT_C_OPTS=\"\n  {}\n\"",
            ac_opts.join("\n  ")
        );
    }

    // Ctrl+F content search widget
    if sk.ctrl_f {
        out.push_str(concat!(
            "skim-file-content-widget() {\n",
            "  local selected file line\n",
            "  selected=$(rg --color=always --line-number --no-heading --smart-case \"${*:-}\" 2>/dev/null |\n",
            "    sk --ansi \\\n",
            "        --delimiter : \\\n",
            "        --preview 'bat --color=always --style=numbers --highlight-line {2} {1}' \\\n",
            "        --preview-window 'up,60%,border-rounded,+{2}+3/3,~3' \\\n",
            "        --header 'Ctrl+F: Search in files | CTRL-/: Toggle Preview')\n",
            "  if [[ -n \"$selected\" ]]; then\n",
            "    file=$(echo \"$selected\" | cut -d: -f1)\n",
            "    line=$(echo \"$selected\" | cut -d: -f2)\n",
            "    ${EDITOR:-nvim} \"+${line}\" \"$file\"\n",
            "  fi\n",
            "}\n",
            "zle -N skim-file-content-widget\n",
            "bindkey '^F' skim-file-content-widget\n",
        ));
    }

    out.push('\n');
}

// ========== Plugins ==========

fn emit_plugins(out: &mut String, config: &ShellConfig) {
    let p = &config.plugins;
    out.push_str("# ===== Plugins =====\n");

    // Direnv (before starship — direnv hook must register first)
    if p.direnv.enabled {
        if !p.direnv.log_format.is_empty() {
            let _ = writeln!(
                out,
                "export DIRENV_LOG_FORMAT=\"{}\"",
                p.direnv.log_format
            );
        } else {
            out.push_str("export DIRENV_LOG_FORMAT=\"\"\n");
        }
        let _ = writeln!(
            out,
            "export DIRENV_WARN_TIMEOUT={}",
            p.direnv.warn_timeout
        );
        out.push_str("eval \"$(direnv hook zsh)\"\n");
    }

    // Zoxide
    if p.zoxide.enabled {
        let _ = writeln!(
            out,
            "eval \"$(zoxide init zsh --cmd {})\"",
            p.zoxide.cmd
        );
        out.push_str("alias zi='__zoxide_zi'\n");
    }

    // Atuin
    if p.atuin.enabled {
        let mut flags = Vec::new();
        if p.atuin.disable_up_arrow {
            flags.push("--disable-up-arrow");
        }
        if p.atuin.disable_ctrl_r {
            flags.push("--disable-ctrl-r");
        }
        let _ = writeln!(
            out,
            "eval \"$(atuin init zsh {})\"",
            flags.join(" ")
        );
    }

    // fzf-tab (must be after compinit, before syntax highlighting)
    if p.fzf_tab.enabled {
        emit_fzf_tab(out, &p.fzf_tab);
    }

    // Starship (should be near the end)
    if p.starship.enabled {
        out.push_str("eval \"$(starship init zsh)\"\n");
    }

    // Syntax highlighting (must be last plugin)
    if p.syntax_highlighting.enabled {
        // The plugin file is sourced from the plugin path set by blzsh/package.nix
        out.push_str(concat!(
            "if [[ -f \"$HOME/.local/share/shell/plugins/zsh-users/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh\" ]]; then\n",
            "  source \"$HOME/.local/share/shell/plugins/zsh-users/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh\"\n",
            "fi\n",
        ));
    }

    out.push('\n');
}

fn emit_fzf_tab(out: &mut String, ft: &config::FzfTabConfig) {
    // Source the plugin
    out.push_str(concat!(
        "if [[ -f \"$HOME/.local/share/shell/plugins/aloxaf/fzf-tab/fzf-tab.plugin.zsh\" ]]; then\n",
        "  source \"$HOME/.local/share/shell/plugins/aloxaf/fzf-tab/fzf-tab.plugin.zsh\"\n",
        "fi\n",
    ));

    let _ = writeln!(
        out,
        "zstyle ':fzf-tab:*' fzf-command {}",
        ft.fzf_command
    );

    if ft.use_fzf_default_opts {
        out.push_str("zstyle ':fzf-tab:*' use-fzf-default-opts yes\n");
    }

    let _ = writeln!(
        out,
        "zstyle ':fzf-tab:*' switch-group '{}' '{}'",
        ft.switch_group[0], ft.switch_group[1]
    );

    if ft.no_sort {
        out.push_str("zstyle ':fzf-tab:*' fzf-flags --no-sort\n");
    }

    // Path scheme for directory-related commands
    for cmd in &ft.path_scheme_commands {
        let _ = writeln!(
            out,
            "zstyle ':fzf-tab:complete:{cmd}:*' fzf-flags --no-sort --scheme=path"
        );
    }

    // Previews
    for (cmd, preview) in &ft.previews {
        let escaped = shell_quote(preview);
        if cmd == "*" {
            let _ = writeln!(
                out,
                "zstyle ':fzf-tab:complete:*:*' fzf-preview '{escaped}'"
            );
        } else {
            let _ = writeln!(
                out,
                "zstyle ':fzf-tab:complete:{cmd}:*' fzf-preview '{escaped}'"
            );
        }
    }

    // Hard-coded advanced previews (process, env, git, systemctl)
    out.push_str(concat!(
        "zstyle ':fzf-tab:complete:(kill|ps):argument-rest' fzf-preview \\\n",
        "  '[[ $group == \"[process ID]\" ]] && ps -p $word -o comm,pid,ppid,%cpu,%mem,start,time,command'\n",
        "zstyle ':fzf-tab:complete:(kill|ps):argument-rest' fzf-flags --preview-window=down:3:wrap\n",
        "zstyle ':fzf-tab:complete:(-command-|-parameter-|-brace-parameter-|export|unset|expand):*' fzf-preview \\\n",
        "  'echo ${(P)word}'\n",
        "zstyle ':fzf-tab:complete:git-(add|diff|restore):*' fzf-preview \\\n",
        "  'git diff $word | delta --width=${FZF_PREVIEW_COLUMNS:-80} 2>/dev/null'\n",
        "zstyle ':fzf-tab:complete:git-log:*' fzf-preview \\\n",
        "  'git log --oneline --graph --color=always $word 2>/dev/null'\n",
        "zstyle ':fzf-tab:complete:git-checkout:*' fzf-preview \\\n",
        "  'case \"$group\" in\n",
        "    \"modified file\") git diff $word | delta --width=${FZF_PREVIEW_COLUMNS:-80} 2>/dev/null ;;\n",
        "    \"recent commit object name\") git log --oneline --graph --color=always $word 2>/dev/null ;;\n",
        "    *) git log --oneline --graph --color=always $word 2>/dev/null ;;\n",
        "  esac'\n",
        "zstyle ':fzf-tab:complete:systemctl-*:*' fzf-preview 'SYSTEMD_COLORS=1 systemctl status $word 2>/dev/null'\n",
    ));
}

// ========== Extra ==========

fn emit_extra(out: &mut String, config: &ShellConfig) {
    if config.extra_zsh.is_empty() {
        return;
    }
    out.push_str("# ===== Extra =====\n");
    for line in &config.extra_zsh {
        let _ = writeln!(out, "{line}");
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    // ===== shell_quote =====

    #[test]
    fn shell_quote_no_quotes() {
        assert_eq!(shell_quote("hello"), "hello");
    }

    #[test]
    fn shell_quote_with_single_quote() {
        assert_eq!(shell_quote("it's"), "it'\\''s");
    }

    #[test]
    fn shell_quote_multiple_single_quotes() {
        assert_eq!(shell_quote("a'b'c"), "a'\\''b'\\''c");
    }

    #[test]
    fn shell_quote_empty_string() {
        assert_eq!(shell_quote(""), "");
    }

    #[test]
    fn shell_quote_only_quote() {
        assert_eq!(shell_quote("'"), "'\\''");
    }

    #[test]
    fn shell_quote_preserves_double_quotes() {
        assert_eq!(shell_quote("he said \"hi\""), "he said \"hi\"");
    }

    #[test]
    fn shell_quote_preserves_backslashes() {
        assert_eq!(shell_quote("path\\to"), "path\\to");
    }

    // ===== validate_config =====

    #[test]
    fn validate_default_config_no_warnings() {
        let config = ShellConfig::default();
        let warnings = validate_config(&config);
        assert!(warnings.is_empty(), "default config should have no warnings: {warnings:?}");
    }

    #[test]
    fn validate_empty_editor_warns() {
        let mut config = ShellConfig::default();
        config.shell.editor = String::new();
        let warnings = validate_config(&config);
        assert!(warnings.iter().any(|w| w.contains("editor")));
    }

    #[test]
    fn validate_zero_history_warns() {
        let mut config = ShellConfig::default();
        config.history.size = 0;
        config.history.save_size = 0;
        let warnings = validate_config(&config);
        assert!(warnings.iter().any(|w| w.contains("history.size")));
        assert!(warnings.iter().any(|w| w.contains("history.save_size")));
    }

    #[test]
    fn validate_high_keytimeout_with_vim_warns() {
        let mut config = ShellConfig::default();
        config.vim_mode.enabled = true;
        config.shell.keytimeout = 200;
        let warnings = validate_config(&config);
        assert!(warnings.iter().any(|w| w.contains("keytimeout")));
    }

    #[test]
    fn validate_high_keytimeout_without_vim_no_warn() {
        let mut config = ShellConfig::default();
        config.vim_mode.enabled = false;
        config.shell.keytimeout = 200;
        let warnings = validate_config(&config);
        assert!(!warnings.iter().any(|w| w.contains("keytimeout")));
    }

    #[test]
    fn validate_fzf_tab_without_default_opts_warns() {
        let mut config = ShellConfig::default();
        config.plugins.fzf_tab.enabled = true;
        config.plugins.fzf_tab.use_fzf_default_opts = false;
        let warnings = validate_config(&config);
        assert!(warnings.iter().any(|w| w.contains("fzf_tab")));
    }

    // ===== generate_zsh — structural =====

    #[test]
    fn generate_zsh_starts_with_header() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.starts_with("# Generated by blx init zsh"));
    }

    #[test]
    fn generate_zsh_contains_all_sections() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("# ===== Shell Options ====="));
        assert!(output.contains("# ===== History ====="));
        assert!(output.contains("# ===== Environment ====="));
        assert!(output.contains("# ===== Completion System ====="));
        assert!(output.contains("# ===== Vim Mode ====="));
        assert!(output.contains("# ===== Aliases ====="));
        assert!(output.contains("# ===== ls() → eza wrapper ====="));
        assert!(output.contains("# ===== Skim (fuzzy finder) ====="));
        assert!(output.contains("# ===== Plugins ====="));
    }

    #[test]
    fn generate_zsh_no_extra_section_when_empty() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(!output.contains("# ===== Extra ====="));
    }

    #[test]
    fn generate_zsh_extra_section_when_populated() {
        let mut config = ShellConfig::default();
        config.extra_zsh = vec!["echo hello".into(), "export FOO=bar".into()];
        let output = generate_zsh(&config);
        assert!(output.contains("# ===== Extra ====="));
        assert!(output.contains("echo hello\n"));
        assert!(output.contains("export FOO=bar\n"));
    }

    // ===== generate_zsh — options =====

    #[test]
    fn options_setopt_when_true() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("setopt AUTO_CD\n"));
        assert!(output.contains("setopt EXTENDED_GLOB\n"));
    }

    #[test]
    fn options_unsetopt_when_false() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        // These are false in defaults
        assert!(output.contains("unsetopt MENU_COMPLETE\n"));
        assert!(output.contains("unsetopt BEEP\n"));
        assert!(output.contains("unsetopt FLOW_CONTROL\n"));
        assert!(output.contains("unsetopt BG_NICE\n"));
    }

    #[test]
    fn options_flip_from_default() {
        let mut config = ShellConfig::default();
        config.options.auto_cd = false;
        config.options.beep = true;
        let output = generate_zsh(&config);
        assert!(output.contains("unsetopt AUTO_CD\n"));
        assert!(output.contains("setopt BEEP\n"));
    }

    // ===== generate_zsh — history =====

    #[test]
    fn history_exports() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("export HISTSIZE=1000000\n"));
        assert!(output.contains("export SAVEHIST=1000000\n"));
        assert!(output.contains("setopt EXTENDED_HISTORY\n"));
        assert!(output.contains("setopt SHARE_HISTORY\n"));
    }

    #[test]
    fn history_custom_values() {
        let mut config = ShellConfig::default();
        config.history.size = 500;
        config.history.save_size = 250;
        config.history.share = false;
        let output = generate_zsh(&config);
        assert!(output.contains("export HISTSIZE=500\n"));
        assert!(output.contains("export SAVEHIST=250\n"));
        assert!(!output.contains("setopt SHARE_HISTORY\n"));
    }

    #[test]
    fn history_mkdir_parent_dir() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("[[ -d \"${HISTFILE:h}\" ]] || mkdir -p \"${HISTFILE:h}\""));
    }

    // ===== generate_zsh — environment =====

    #[test]
    fn environment_exports_editor() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("export EDITOR='blnvim'\n"));
        assert!(output.contains("export VISUAL='blnvim'\n"));
        assert!(output.contains("export PAGER='less'\n"));
    }

    #[test]
    fn environment_shell_quotes_values() {
        let mut config = ShellConfig::default();
        config.shell.editor = "nvim -c 'set noswap'".into();
        let output = generate_zsh(&config);
        assert!(output.contains("export EDITOR='nvim -c '\\''set noswap'\\'''"));
    }

    #[test]
    fn environment_user_vars() {
        let mut config = ShellConfig::default();
        config.environment.insert("MY_VAR".into(), "hello".into());
        config.environment.insert("QUOTED".into(), "it's a test".into());
        let output = generate_zsh(&config);
        assert!(output.contains("export MY_VAR='hello'\n"));
        assert!(output.contains("export QUOTED='it'\\''s a test'\n"));
    }

    #[test]
    fn environment_manpager_single_quoted() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("export MANPAGER='sh -c '\\''col -bx | bat -l man -p'\\'''"));
    }

    // ===== generate_zsh — vim mode =====

    #[test]
    fn vim_mode_enabled() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("bindkey -v\n"));
        assert!(output.contains("zle-keymap-select"));
        assert!(output.contains("edit-command-line"));
        assert!(output.contains("vi-yank-clip"));
    }

    #[test]
    fn vim_mode_disabled() {
        let mut config = ShellConfig::default();
        config.vim_mode.enabled = false;
        let output = generate_zsh(&config);
        assert!(output.contains("bindkey -e\n"));
        assert!(!output.contains("bindkey -v\n"));
        assert!(!output.contains("zle-keymap-select"));
    }

    #[test]
    fn vim_mode_no_cursor_shape() {
        let mut config = ShellConfig::default();
        config.vim_mode.cursor_shape = false;
        let output = generate_zsh(&config);
        assert!(output.contains("bindkey -v\n"));
        assert!(!output.contains("zle-keymap-select"));
    }

    #[test]
    fn vim_mode_no_clipboard() {
        let mut config = ShellConfig::default();
        config.vim_mode.system_clipboard = false;
        let output = generate_zsh(&config);
        assert!(output.contains("bindkey -v\n"));
        assert!(!output.contains("vi-yank-clip"));
    }

    #[test]
    fn vim_mode_no_edit_command_line() {
        let mut config = ShellConfig::default();
        config.vim_mode.edit_command_line = false;
        let output = generate_zsh(&config);
        assert!(!output.contains("edit-command-line"));
    }

    // ===== generate_zsh — aliases =====

    #[test]
    fn aliases_emitted() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("alias gs='git status'\n"));
        assert!(output.contains("alias cat='bat --style=plain --paging=never'\n"));
    }

    #[test]
    fn aliases_shell_quoted() {
        let mut config = ShellConfig::default();
        config.aliases.clear();
        config.aliases.insert("test".into(), "echo 'hello world'".into());
        let output = generate_zsh(&config);
        assert!(output.contains("alias test='echo '\\''hello world'\\'''"));
    }

    #[test]
    fn platform_aliases_always_emitted() {
        let mut config = ShellConfig::default();
        config.aliases.clear();
        let output = generate_zsh(&config);
        // Platform aliases must appear even with empty alias map
        assert!(output.contains("alias pbcopy="));
        assert!(output.contains("alias ports="));
    }

    // ===== generate_zsh — eza wrapper =====

    #[test]
    fn eza_wrapper_present() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("ls() {"));
        assert!(output.contains("eza_args=(--icons --group-directories-first)"));
        assert!(output.contains("alias l='eza --icons --group-directories-first'"));
    }

    #[test]
    fn eza_wrapper_disabled() {
        let mut config = ShellConfig::default();
        config.tools.eza.ls_wrapper = false;
        let output = generate_zsh(&config);
        assert!(!output.contains("ls() {"));
        assert!(!output.contains("# ===== ls() → eza wrapper ====="));
    }

    #[test]
    fn eza_no_icons() {
        let mut config = ShellConfig::default();
        config.tools.eza.icons = false;
        let output = generate_zsh(&config);
        // eza base_flags should not contain --icons (but skim previews still reference eza --icons)
        assert!(output.contains("eza_args=(--group-directories-first)"));
        assert!(!output.contains("eza_args=(--icons"));
        assert!(output.contains("alias l='eza --group-directories-first'"));
    }

    // ===== generate_zsh — skim =====

    #[test]
    fn skim_default_options() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("export SKIM_DEFAULT_OPTIONS="));
        assert!(output.contains("--algo=arinae"));
        assert!(output.contains("--layout=reverse"));
        assert!(output.contains("--ansi"));
    }

    #[test]
    fn skim_fzf_bridge() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("export FZF_DEFAULT_OPTS=\"$SKIM_DEFAULT_OPTIONS\""));
    }

    #[test]
    fn skim_ctrl_t_command_single_quoted() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("export SKIM_CTRL_T_COMMAND='fd --type f"));
    }

    #[test]
    fn skim_alt_c_command_single_quoted() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("export SKIM_ALT_C_COMMAND='fd --type d"));
    }

    #[test]
    fn skim_ctrl_t_disabled() {
        let mut config = ShellConfig::default();
        config.tools.skim.ctrl_t.enabled = false;
        let output = generate_zsh(&config);
        assert!(!output.contains("SKIM_CTRL_T_COMMAND"));
        assert!(!output.contains("SKIM_CTRL_T_OPTS"));
    }

    #[test]
    fn skim_alt_c_disabled() {
        let mut config = ShellConfig::default();
        config.tools.skim.alt_c.enabled = false;
        let output = generate_zsh(&config);
        assert!(!output.contains("SKIM_ALT_C_COMMAND"));
        assert!(!output.contains("SKIM_ALT_C_OPTS"));
    }

    #[test]
    fn skim_ctrl_f_disabled() {
        let mut config = ShellConfig::default();
        config.tools.skim.ctrl_f = false;
        let output = generate_zsh(&config);
        assert!(!output.contains("skim-file-content-widget"));
    }

    #[test]
    fn skim_ctrl_f_enabled() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("skim-file-content-widget"));
        assert!(output.contains("bindkey '^F'"));
    }

    // ===== generate_zsh — plugins =====

    #[test]
    fn plugins_direnv() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("eval \"$(direnv hook zsh)\""));
        assert!(output.contains("DIRENV_LOG_FORMAT"));
    }

    #[test]
    fn plugins_zoxide() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("eval \"$(zoxide init zsh --cmd cd)\""));
        assert!(output.contains("alias zi='__zoxide_zi'"));
    }

    #[test]
    fn plugins_zoxide_custom_cmd() {
        let mut config = ShellConfig::default();
        config.plugins.zoxide.cmd = "j".into();
        let output = generate_zsh(&config);
        assert!(output.contains("eval \"$(zoxide init zsh --cmd j)\""));
    }

    #[test]
    fn plugins_atuin() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("eval \"$(atuin init zsh --disable-up-arrow --disable-ctrl-r)\""));
    }

    #[test]
    fn plugins_atuin_no_flags() {
        let mut config = ShellConfig::default();
        config.plugins.atuin.disable_up_arrow = false;
        config.plugins.atuin.disable_ctrl_r = false;
        let output = generate_zsh(&config);
        assert!(output.contains("eval \"$(atuin init zsh )\""));
    }

    #[test]
    fn plugins_starship() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("eval \"$(starship init zsh)\""));
    }

    #[test]
    fn plugins_starship_disabled() {
        let mut config = ShellConfig::default();
        config.plugins.starship.enabled = false;
        let output = generate_zsh(&config);
        assert!(!output.contains("starship init"));
    }

    #[test]
    fn plugins_fzf_tab() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("fzf-tab.plugin.zsh"));
        assert!(output.contains("fzf-command skim-tab"));
        assert!(output.contains("use-fzf-default-opts yes"));
    }

    #[test]
    fn plugins_syntax_highlighting_last() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        let starship_pos = output.find("starship init").unwrap();
        let sh_pos = output.find("zsh-syntax-highlighting.zsh").unwrap();
        assert!(sh_pos > starship_pos, "syntax highlighting must come after starship");
    }

    #[test]
    fn plugins_direnv_before_starship() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        let direnv_pos = output.find("direnv hook").unwrap();
        let starship_pos = output.find("starship init").unwrap();
        assert!(direnv_pos < starship_pos, "direnv must come before starship");
    }

    #[test]
    fn plugins_all_disabled() {
        let mut config = ShellConfig::default();
        config.plugins.direnv.enabled = false;
        config.plugins.zoxide.enabled = false;
        config.plugins.atuin.enabled = false;
        config.plugins.fzf_tab.enabled = false;
        config.plugins.starship.enabled = false;
        config.plugins.syntax_highlighting.enabled = false;
        let output = generate_zsh(&config);
        assert!(!output.contains("direnv"));
        assert!(!output.contains("zoxide"));
        assert!(!output.contains("atuin"));
        assert!(!output.contains("fzf-tab"));
        assert!(!output.contains("starship"));
        assert!(!output.contains("syntax-highlighting"));
    }

    // ===== generate_zsh — completion =====

    #[test]
    fn completion_compinit() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("autoload -Uz compinit"));
        assert!(output.contains("compinit"));
    }

    #[test]
    fn completion_matcher_list() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("matcher-list"));
        assert!(output.contains("m:{a-zA-Z}={A-Za-z}"));
    }

    #[test]
    fn completion_no_matchers_when_all_disabled() {
        let mut config = ShellConfig::default();
        config.completion.case_insensitive = false;
        config.completion.partial_word = false;
        config.completion.substring = false;
        let output = generate_zsh(&config);
        assert!(!output.contains("matcher-list"));
    }

    #[test]
    fn completion_special_dirs() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("special-dirs true"));
    }

    #[test]
    fn completion_rehash() {
        let config = ShellConfig::default();
        let output = generate_zsh(&config);
        assert!(output.contains("rehash true"));
    }

    // ===== generate_zsh — roundtrip =====

    #[test]
    fn default_config_generates_deterministic_output() {
        let config = ShellConfig::default();
        let output1 = generate_zsh(&config);
        let output2 = generate_zsh(&config);
        assert_eq!(output1, output2);
    }

    #[test]
    fn yaml_roundtrip_generates_same_zsh() {
        let config1 = ShellConfig::default();
        let yaml = serde_yaml_ng::to_string(&config1).unwrap();
        let config2 = config::parse_config(&yaml).unwrap();
        let zsh1 = generate_zsh(&config1);
        let zsh2 = generate_zsh(&config2);
        assert_eq!(zsh1, zsh2);
    }

    #[test]
    fn partial_config_generates_valid_output() {
        let yaml = "shell:\n  editor: vim\nplugins:\n  starship:\n    enabled: false\n";
        let config = config::parse_config(yaml).unwrap();
        let output = generate_zsh(&config);
        assert!(output.contains("export EDITOR='vim'"));
        assert!(!output.contains("starship init"));
        // Everything else should still be present
        assert!(output.contains("setopt AUTO_CD"));
        assert!(output.contains("autoload -Uz compinit"));
    }

    // ===== shell injection safety =====

    #[test]
    fn env_var_value_with_single_quote_is_safe() {
        let mut config = ShellConfig::default();
        config.environment.insert("DANGER".into(), "'; rm -rf /; echo '".into());
        let output = generate_zsh(&config);
        // The leading quote must be escaped — the shell sees:
        //   export DANGER=''\''; rm -rf /; echo '\'''
        // which assigns the literal string, not executing rm.
        // Verify the opening quote is escaped (not left bare to close the value early)
        assert!(output.contains("export DANGER=''\\''"));
        assert!(output.contains("'\\''"));
    }

    #[test]
    fn alias_value_with_single_quote_is_safe() {
        let mut config = ShellConfig::default();
        config.aliases.clear();
        config.aliases.insert("safe".into(), "echo 'hello'; echo 'world'".into());
        let output = generate_zsh(&config);
        assert!(output.contains("alias safe='echo '\\''hello'\\''"));
    }

    #[test]
    fn editor_with_single_quote_is_safe() {
        let mut config = ShellConfig::default();
        config.shell.editor = "nvim -c 'set noswap'".into();
        let output = generate_zsh(&config);
        // Must escape the embedded single quote
        assert!(output.contains("'\\''set noswap'\\''"));
    }

    #[test]
    fn skim_ctrl_t_command_with_single_quote_is_safe() {
        let mut config = ShellConfig::default();
        config.tools.skim.ctrl_t.command = "fd --type f --exclude 'node_modules'".into();
        let output = generate_zsh(&config);
        assert!(output.contains("SKIM_CTRL_T_COMMAND="));
        assert!(output.contains("'\\''node_modules'\\''"));
    }

    #[test]
    fn skim_alt_c_command_with_single_quote_is_safe() {
        let mut config = ShellConfig::default();
        config.tools.skim.alt_c.command = "fd --type d --exclude 'node_modules'".into();
        let output = generate_zsh(&config);
        assert!(output.contains("SKIM_ALT_C_COMMAND="));
        assert!(output.contains("'\\''node_modules'\\''"));
    }
}
