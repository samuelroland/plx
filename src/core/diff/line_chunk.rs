use console::Style;

#[derive(Debug, PartialEq, Eq, Clone)]
//Represents a combination of words in the same line
pub(super) struct LineChunk {
    is_different: bool,
    value: String,
}

impl LineChunk {
    pub fn new(value: String, is_different: bool) -> Self {
        Self {
            value,
            is_different,
        }
    }
    pub(super) fn to_ansi_colors(&self, style: &Style) -> String {
        if self.is_different {
            format!("{}", style.apply_to(&self.value).bold())
        } else {
            format!("{}", style.apply_to(&self.value).dim())
        }
    }

    pub(super) fn to_html(&self) -> String {
        if self.is_different {
            format!("<span class='diff-bold'>{}</span>", self.value)
        } else {
            format!("<span>{}</span>", self.value)
        }
    }
}
