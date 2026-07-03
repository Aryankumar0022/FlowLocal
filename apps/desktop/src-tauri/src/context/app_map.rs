// ============================================================
// context/app_map.rs — Map process names → context labels
//
// The context label drives which LLM prompt template is used
// for cleanup and formatting.
// ============================================================

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Human-readable context label used for prompt template selection.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppContext {
    /// VS Code, Cursor, Neovim, etc.
    Code,
    /// Terminal / shell applications
    Terminal,
    /// Email clients
    Email,
    /// Chat / messaging apps (Slack, Discord, Teams)
    Chat,
    /// Note-taking apps (Notion, Obsidian, Logseq)
    Notes,
    /// Web browsers (Chrome, Firefox, Edge)
    Browser,
    /// Document editors (Word, Google Docs, LibreOffice)
    Document,
    /// Spreadsheet apps
    Spreadsheet,
    /// Video conferencing (Zoom, Meet, Teams)
    Meeting,
    /// Catch-all
    Generic,
}

impl AppContext {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Terminal => "terminal",
            Self::Email => "email",
            Self::Chat => "chat",
            Self::Notes => "notes",
            Self::Browser => "browser",
            Self::Document => "document",
            Self::Spreadsheet => "spreadsheet",
            Self::Meeting => "meeting",
            Self::Generic => "generic",
        }
    }
}

/// Maps known process names (lowercase) to context labels.
static PROCESS_MAP: Lazy<HashMap<&'static str, AppContext>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // ── Code editors ──────────────────────────────────────────
    m.insert("code", AppContext::Code);           // VS Code
    m.insert("code.exe", AppContext::Code);
    m.insert("cursor", AppContext::Code);         // Cursor
    m.insert("cursor.exe", AppContext::Code);
    m.insert("sublime_text", AppContext::Code);
    m.insert("sublime_text.exe", AppContext::Code);
    m.insert("atom", AppContext::Code);
    m.insert("atom.exe", AppContext::Code);
    m.insert("notepad++", AppContext::Code);
    m.insert("notepad++.exe", AppContext::Code);
    m.insert("pycharm64", AppContext::Code);
    m.insert("pycharm64.exe", AppContext::Code);
    m.insert("idea64", AppContext::Code);
    m.insert("idea64.exe", AppContext::Code);
    m.insert("clion64", AppContext::Code);
    m.insert("nvim", AppContext::Code);
    m.insert("vim", AppContext::Code);
    m.insert("emacs", AppContext::Code);
    m.insert("zed", AppContext::Code);

    // ── Terminals ─────────────────────────────────────────────
    m.insert("windowsterminal", AppContext::Terminal);
    m.insert("windowsterminal.exe", AppContext::Terminal);
    m.insert("cmd", AppContext::Terminal);
    m.insert("cmd.exe", AppContext::Terminal);
    m.insert("powershell", AppContext::Terminal);
    m.insert("powershell.exe", AppContext::Terminal);
    m.insert("pwsh", AppContext::Terminal);
    m.insert("pwsh.exe", AppContext::Terminal);
    m.insert("bash", AppContext::Terminal);
    m.insert("zsh", AppContext::Terminal);
    m.insert("fish", AppContext::Terminal);
    m.insert("alacritty", AppContext::Terminal);
    m.insert("kitty", AppContext::Terminal);
    m.insert("wezterm", AppContext::Terminal);
    m.insert("gnome-terminal", AppContext::Terminal);
    m.insert("konsole", AppContext::Terminal);
    m.insert("xterm", AppContext::Terminal);
    m.insert("iterm2", AppContext::Terminal);
    m.insert("hyper", AppContext::Terminal);

    // ── Email ─────────────────────────────────────────────────
    m.insert("thunderbird", AppContext::Email);
    m.insert("thunderbird.exe", AppContext::Email);
    m.insert("outlook", AppContext::Email);
    m.insert("outlook.exe", AppContext::Email);
    m.insert("mailspring", AppContext::Email);

    // ── Chat / messaging ──────────────────────────────────────
    m.insert("slack", AppContext::Chat);
    m.insert("slack.exe", AppContext::Chat);
    m.insert("discord", AppContext::Chat);
    m.insert("discord.exe", AppContext::Chat);
    m.insert("teams", AppContext::Chat);
    m.insert("teams.exe", AppContext::Chat);
    m.insert("msteams", AppContext::Chat);
    m.insert("msteams.exe", AppContext::Chat);
    m.insert("telegram-desktop", AppContext::Chat);
    m.insert("telegram.exe", AppContext::Chat);
    m.insert("signal", AppContext::Chat);
    m.insert("signal.exe", AppContext::Chat);
    m.insert("whatsapp", AppContext::Chat);

    // ── Notes ─────────────────────────────────────────────────
    m.insert("notion", AppContext::Notes);
    m.insert("notion.exe", AppContext::Notes);
    m.insert("obsidian", AppContext::Notes);
    m.insert("obsidian.exe", AppContext::Notes);
    m.insert("logseq", AppContext::Notes);
    m.insert("logseq.exe", AppContext::Notes);
    m.insert("roam", AppContext::Notes);
    m.insert("notepad", AppContext::Notes);
    m.insert("notepad.exe", AppContext::Notes);
    m.insert("onenote", AppContext::Notes);

    // ── Browsers ──────────────────────────────────────────────
    m.insert("chrome", AppContext::Browser);
    m.insert("chrome.exe", AppContext::Browser);
    m.insert("firefox", AppContext::Browser);
    m.insert("firefox.exe", AppContext::Browser);
    m.insert("msedge", AppContext::Browser);
    m.insert("msedge.exe", AppContext::Browser);
    m.insert("brave", AppContext::Browser);
    m.insert("brave.exe", AppContext::Browser);
    m.insert("safari", AppContext::Browser);
    m.insert("opera", AppContext::Browser);

    // ── Documents ─────────────────────────────────────────────
    m.insert("winword", AppContext::Document);
    m.insert("winword.exe", AppContext::Document);
    m.insert("soffice", AppContext::Document);
    m.insert("soffice.exe", AppContext::Document);
    m.insert("wps", AppContext::Document);

    // ── Spreadsheets ──────────────────────────────────────────
    m.insert("excel", AppContext::Spreadsheet);
    m.insert("excel.exe", AppContext::Spreadsheet);
    m.insert("scalc", AppContext::Spreadsheet);

    // ── Video conferencing ────────────────────────────────────
    m.insert("zoom", AppContext::Meeting);
    m.insert("zoom.exe", AppContext::Meeting);
    m.insert("meet", AppContext::Meeting);

    m
});

/// Look up the context label for a given process name.
/// Falls back to `AppContext::Generic` if unknown.
pub fn map_process(process_name: &str) -> AppContext {
    let key = process_name.to_lowercase();
    PROCESS_MAP
        .get(key.as_str())
        .cloned()
        .unwrap_or(AppContext::Generic)
}

/// Infer context from the window title when the process is a browser
/// (because Chrome hosts Gmail, Notion, and Slack all under the same process).
pub fn refine_browser_context(window_title: &str) -> AppContext {
    let title = window_title.to_lowercase();
    if title.contains("gmail") || title.contains("mail.google") || title.contains("outlook.com") {
        AppContext::Email
    } else if title.contains("notion") {
        AppContext::Notes
    } else if title.contains("slack") {
        AppContext::Chat
    } else if title.contains("discord") {
        AppContext::Chat
    } else if title.contains("docs.google") || title.contains("document") {
        AppContext::Document
    } else if title.contains("sheets.google") || title.contains("spreadsheet") {
        AppContext::Spreadsheet
    } else if title.contains("meet.google") || title.contains("zoom.us") {
        AppContext::Meeting
    } else {
        AppContext::Browser
    }
}
