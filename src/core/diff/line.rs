use console::Style;

use super::{diff_type::DiffType, line_chunk::LineChunk};

#[derive(Debug, PartialEq, Eq, Clone)]
//Represents a Line in a difference
pub(super) struct Line {
    line_chunks: Vec<LineChunk>,
    missing_new_line: bool,
    difference_type: DiffType,
}
impl Line {
    pub(super) fn new(
        line_chunks: Vec<LineChunk>,
        missing_new_line: bool,
        difference_type: DiffType,
    ) -> Self {
        Self {
            line_chunks,
            missing_new_line,
            difference_type,
        }
    }
    pub(super) fn to_ansi_colors(&self) -> String {
        let mut result = String::new();
        let (sign, s) = match self.difference_type {
            DiffType::Removed => ("-", Style::new().red().bold()),
            DiffType::Added => ("+", Style::new().green().bold()),
            DiffType::NoDiff => (" ", Style::new().dim()),
        };

        result.push_str(&format!("{}", s.apply_to(sign)));
        for chunk in &self.line_chunks {
            result.push_str(&chunk.to_ansi_colors(&s));
        }
        if self.missing_new_line {
            result.push('\n');
        }
        result
    }

    pub(super) fn to_html(&self) -> String {
        let mut result = String::new();
        let (sign, diff_css_class) = match self.difference_type {
            DiffType::Removed => ("-", "diff-minus"),
            DiffType::Added => ("+", "diff-plus"),
            DiffType::NoDiff => (" ", "diff-none"),
        };

        let mut line = String::new();
        for chunk in &self.line_chunks {
            line.push_str(&chunk.to_html(diff_css_class));
        }
        result.push_str(&format!("<span>{sign} {line}</span>"));
        if self.missing_new_line {
            result.push('\n'); // only add a \n not a <br> because we display it in <pre>
        }
        result
    }
}
