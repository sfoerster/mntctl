use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;

/// Format a mount status string with appropriate coloring.
pub fn status_style(status: &str) -> String {
    match status {
        "mounted" | "active" => status.if_supports_color(Stdout, |s| s.green()).to_string(),
        "unmounted" | "inactive" | "stopped" => {
            status.if_supports_color(Stdout, |s| s.red()).to_string()
        }
        "enabled" => status
            .if_supports_color(Stdout, |s| s.bright_green())
            .to_string(),
        "disabled" => status.if_supports_color(Stdout, |s| s.yellow()).to_string(),
        "error" | "failed" => status
            .if_supports_color(Stdout, |s| s.bright_red())
            .to_string(),
        _ => status.to_string(),
    }
}

/// Format a label (e.g., column header) in bold.
pub fn label_style(label: &str) -> String {
    label.if_supports_color(Stdout, |s| s.bold()).to_string()
}

/// Format a name/identifier in cyan.
pub fn name_style(name: &str) -> String {
    name.if_supports_color(Stdout, |s| s.cyan()).to_string()
}

/// Format a success message.
pub fn success(msg: &str) -> String {
    msg.if_supports_color(Stdout, |s| s.green()).to_string()
}

/// Format an error message.
pub fn error(msg: &str) -> String {
    msg.if_supports_color(Stdout, |s| s.red()).to_string()
}

/// Format an info/notice message.
pub fn info(msg: &str) -> String {
    msg.if_supports_color(Stdout, |s| s.blue()).to_string()
}
