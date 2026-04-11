use crate::domain::role::RoleId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub email: String,
    pub location: Option<String>,
    pub raw_text: String,
    pub analysis: Option<ProfileAnalysis>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileAnalysis {
    pub summary: String,
    pub primary_role: RoleId,
    pub seniority: String,
    pub skills: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateProfile {
    pub name: String,
    pub email: String,
    pub location: Option<String>,
    pub raw_text: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpdateProfile {
    pub name: Option<String>,
    pub email: Option<String>,
    pub location: Option<Option<String>>,
    pub raw_text: Option<String>,
}
