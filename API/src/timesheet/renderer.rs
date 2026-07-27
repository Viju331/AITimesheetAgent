use super::models::{HeadingStyle, StyleProfile, TaskGroupDraft};

/// Render the styled draft as plain text, respecting the user's learned
/// heading and bullet conventions (P5-013).
pub fn to_plain_text(groups: &[TaskGroupDraft], style: &StyleProfile) -> String {
    let mut out = String::new();
    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format_heading(&group.title, style.heading_style));
        out.push('\n');
        for (bi, bullet) in group.bullets.iter().enumerate() {
            out.push_str(&style.bullet_char.as_prefix(bi));
            out.push_str(&bullet.text);
            out.push('\n');
        }
    }
    out.trim_end().to_string()
}

/// Render as GitHub-flavored markdown — headings are always `##` regardless
/// of the user's plain-text heading style, since markdown has its own convention.
pub fn to_markdown(groups: &[TaskGroupDraft], style: &StyleProfile) -> String {
    let mut out = String::new();
    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str("## ");
        out.push_str(&group.title);
        if let Some(ticket) = &group.ticket_reference {
            out.push_str(&format!(" ({ticket})"));
        }
        out.push('\n');
        for (bi, bullet) in group.bullets.iter().enumerate() {
            out.push_str(&style.bullet_char.as_prefix(bi));
            out.push_str(&bullet.text);
            out.push('\n');
        }
    }
    out.trim_end().to_string()
}

/// Render as HTML for an in-app rich-text preview.
pub fn to_html(groups: &[TaskGroupDraft]) -> String {
    let mut out = String::new();
    for group in groups {
        out.push_str("<h3>");
        out.push_str(&escape_html(&group.title));
        if let Some(ticket) = &group.ticket_reference {
            out.push_str(&format!(" <small>({})</small>", escape_html(ticket)));
        }
        out.push_str("</h3><ul>");
        for bullet in &group.bullets {
            out.push_str("<li>");
            out.push_str(&escape_html(&bullet.text));
            out.push_str("</li>");
        }
        out.push_str("</ul>");
    }
    out
}

fn format_heading(title: &str, style: HeadingStyle) -> String {
    match style {
        HeadingStyle::Dash => format!("Task - {title}"),
        HeadingStyle::Colon => format!("{title}:"),
        HeadingStyle::Brackets => format!("[{title}]"),
        HeadingStyle::Bold => format!("**{title}**"),
        HeadingStyle::Plain => title.to_string(),
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timesheet::models::BulletPoint;

    fn groups() -> Vec<TaskGroupDraft> {
        vec![TaskGroupDraft {
            title: "Admin Screen".to_string(),
            task_key: "admin screen".to_string(),
            ticket_reference: Some("JIRA-1".to_string()),
            project_id: None,
            bullets: vec![BulletPoint { text: "Fixed dropdown".to_string(), source_activity_ids: vec![], confidence_score: 0.5 }],
        }]
    }

    #[test]
    fn renders_plain_text_with_dash_heading() {
        let text = to_plain_text(&groups(), &StyleProfile::default());
        assert_eq!(text, "Task - Admin Screen\n- Fixed dropdown");
    }

    #[test]
    fn renders_markdown_with_heading_level_two() {
        let md = to_markdown(&groups(), &StyleProfile::default());
        assert!(md.starts_with("## Admin Screen (JIRA-1)"));
        assert!(md.contains("- Fixed dropdown"));
    }

    #[test]
    fn renders_html_with_escaped_content() {
        let mut g = groups();
        g[0].bullets[0].text = "Fixed <script> issue".to_string();
        let html = to_html(&g);
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("<h3>Admin Screen"));
    }
}
