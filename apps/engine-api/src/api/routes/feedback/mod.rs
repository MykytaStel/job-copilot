mod handlers;
#[cfg(test)]
mod tests;

pub(crate) use handlers::ensure_profile_exists;
pub use handlers::{
    add_company_blacklist, add_company_whitelist, clear_all_hidden_jobs, hide_job, list_feedback,
    mark_job_bad_fit, remove_company_blacklist, remove_company_whitelist, save_job,
    set_job_interest_rating, set_job_legitimacy_signal, set_job_salary_signal,
    set_job_work_mode_signal, tag_job_feedback, unhide_job, unmark_job_bad_fit, unsave_job,
};
