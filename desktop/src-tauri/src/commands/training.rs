use plx::live::msg::CheckStatus;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
struct File {
    path: String,
    content: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, Type)]
struct Answer {
    files: Vec<File>,
    checks_status: Vec<CheckStatus>,
}
