use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::domain::user_event::model::{UserEventRecord, UserEventType};

pub fn outcome_job_ids(events: &[UserEventRecord], feedback_job_ids: &[String]) -> Vec<String> {
    let mut job_ids = BTreeSet::new();

    for event in events {
        if !matches!(
            event.event_type,
            UserEventType::JobOpened
                | UserEventType::JobSaved
                | UserEventType::JobHidden
                | UserEventType::JobBadFit
                | UserEventType::ApplicationCreated
        ) {
            continue;
        }

        if let Some(job_id) = normalized_job_id(event.job_id.as_deref()) {
            job_ids.insert(job_id);
        }
    }

    for job_id in feedback_job_ids {
        if let Some(job_id) = normalized_job_id(Some(job_id.as_str())) {
            job_ids.insert(job_id);
        }
    }

    job_ids.into_iter().collect()
}

pub(crate) fn application_ids_by_job_id(events: &[UserEventRecord]) -> BTreeMap<String, String> {
    let mut ordered_events = events
        .iter()
        .filter(|event| matches!(event.event_type, UserEventType::ApplicationCreated))
        .filter_map(|event| {
            let job_id = normalized_job_id(event.job_id.as_deref())?;
            let application_id = application_id_from_payload(event.payload_json.as_ref())?;
            Some((job_id, application_id, &event.created_at, &event.id))
        })
        .collect::<Vec<_>>();
    ordered_events.sort_by(
        |(left_job_id, _, left_created_at, left_id),
         (right_job_id, _, right_created_at, right_id)| {
            left_job_id
                .cmp(right_job_id)
                .then_with(|| left_created_at.cmp(right_created_at))
                .then_with(|| left_id.cmp(right_id))
        },
    );

    let mut application_ids = BTreeMap::new();
    for (job_id, application_id, _, _) in ordered_events {
        application_ids.insert(job_id, application_id);
    }

    application_ids
}

pub(super) fn normalized_job_id(job_id: Option<&str>) -> Option<String> {
    job_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn application_id_from_payload(payload: Option<&Value>) -> Option<String> {
    payload
        .and_then(|value| value.get("application_id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
