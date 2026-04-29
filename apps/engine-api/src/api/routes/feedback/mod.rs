mod handlers;
#[cfg(test)]
mod tests;

pub(crate) use handlers::ensure_profile_exists;
#[cfg(test)]
pub use handlers::{
    BulkHideJobsByCompanyQuery, ExportFeedbackQuery, FeedbackStatsQuery, FeedbackTimelineQuery,
    JobFeedbackActionQuery, RemoveCompanyBlacklistBySlugQuery, UpdateCompanyFeedbackBySlugQuery,
};
pub use handlers::{
    add_company_blacklist, add_company_whitelist, bulk_hide_jobs_by_company, clear_all_hidden_jobs,
    export_feedback_csv, get_feedback_stats, hide_job, list_feedback, list_feedback_timeline,
    mark_job_bad_fit, mark_job_bad_fit_by_query, patch_job_interest_rating_by_query,
    remove_company_blacklist, remove_company_blacklist_by_slug, remove_company_whitelist, save_job,
    set_job_interest_rating, set_job_legitimacy_signal, set_job_salary_signal,
    set_job_work_mode_signal, tag_job_feedback, undo_job_bad_fit, undo_job_hide, unhide_job,
    unmark_job_bad_fit, unsave_job, update_company_feedback_notes_by_slug,
};
