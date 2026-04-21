mod handlers;
#[cfg(test)]
mod tests;

pub(crate) use handlers::ensure_profile_exists;
pub use handlers::{
    add_company_blacklist, add_company_whitelist, hide_job, list_feedback, mark_job_bad_fit,
    remove_company_blacklist, remove_company_whitelist, save_job, unhide_job, unmark_job_bad_fit,
    unsave_job,
};
