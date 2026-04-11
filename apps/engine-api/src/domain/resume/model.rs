#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResumeVersion {
    pub id: String,
    pub version: i32,
    pub filename: String,
    pub raw_text: String,
    pub is_active: bool,
    pub uploaded_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadResume {
    pub filename: String,
    pub raw_text: String,
}
