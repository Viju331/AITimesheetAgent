use std::io::Read;
use std::path::Path;
use regex::Regex;
use std::sync::OnceLock;
use crate::errors::{AppError, AppResult};

fn docx_text_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"<w:t[^>]*>(.*?)</w:t>").unwrap())
}

fn markdown_syntax_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"^#{1,6}\s*|\*\*|__|`").unwrap())
}

/// Extract plain text from an uploaded style-example file. Supports `.txt`,
/// `.md` (markdown syntax stripped), and `.docx` (extracted from the
/// `word/document.xml` part inside the zip). `.pdf` is not yet supported —
/// callers should surface a clear message rather than fail silently.
pub fn extract_text(path: &Path) -> AppResult<String> {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    match extension.as_str() {
        "txt" => std::fs::read_to_string(path).map_err(AppError::from),
        "md" | "markdown" => {
            let raw = std::fs::read_to_string(path)?;
            Ok(strip_markdown(&raw))
        }
        "docx" => extract_docx_text(path),
        "pdf" => Err(AppError::Parse(
            "PDF style examples aren't supported yet — please upload .txt, .md, or .docx".to_string(),
        )),
        other => Err(AppError::Parse(format!("Unsupported style example file type: .{other}"))),
    }
}

fn strip_markdown(raw: &str) -> String {
    raw.lines().map(|line| markdown_syntax_pattern().replace_all(line, "").trim_start().to_string()).collect::<Vec<_>>().join("\n")
}

fn extract_docx_text(path: &Path) -> AppResult<String> {
    let file = std::fs::File::open(path).map_err(AppError::from)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| AppError::Parse(format!("invalid .docx archive: {e}")))?;

    let mut document_xml = String::new();
    {
        let mut entry = archive
            .by_name("word/document.xml")
            .map_err(|e| AppError::Parse(format!("missing word/document.xml: {e}")))?;
        entry.read_to_string(&mut document_xml).map_err(AppError::from)?;
    }

    let with_breaks = document_xml.replace("</w:p>", "\n").replace("<w:br/>", "\n");

    let mut text = String::new();
    for cap in docx_text_pattern().captures_iter(&with_breaks) {
        text.push_str(&cap[1]);
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_markdown_headings_and_emphasis() {
        let raw = "## Task - Admin Screen\n- **Implemented** dropdown\n- `Fixed` bug";
        let result = strip_markdown(raw);
        assert_eq!(result, "Task - Admin Screen\n- Implemented dropdown\n- Fixed bug");
    }

    #[test]
    fn extracts_text_from_docx_xml_fragment() {
        let with_breaks = "<w:p><w:r><w:t>Implemented dropdown</w:t></w:r></w:p>\n<w:p><w:r><w:t>Fixed bug</w:t></w:r></w:p>\n";
        let mut text = String::new();
        for cap in docx_text_pattern().captures_iter(with_breaks) {
            text.push_str(&cap[1]);
        }
        assert_eq!(text, "Implemented dropdownFixed bug");
    }
}
