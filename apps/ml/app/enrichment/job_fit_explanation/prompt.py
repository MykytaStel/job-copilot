import json

from app.enrichment.job_fit_explanation.contract import (
    JobFitExplanationPrompt,
    JobFitExplanationRequest,
    job_fit_explanation_schema,
)


def build_job_fit_explanation_prompt(
    context: JobFitExplanationRequest,
) -> JobFitExplanationPrompt:
    prompt_context = context.model_dump(exclude={"profile_id"}, by_alias=True)
    output_schema = job_fit_explanation_schema()

    return JobFitExplanationPrompt(
        system_instructions=(
            "You generate additive job-fit explanations for a job search copilot. "
            "Use only the deterministic context provided. "
            "Do not change or reinterpret ranking, score, canonical IDs, source IDs, or entities. "
            "Explain why the ranked job looks like a good fit, weak fit, or risky fit based on the provided profile, search profile, deterministic fit evidence, job payload, and feedback signals. "
            "If a signal is absent or ambiguous, put it in missing_signals instead of inventing it. "
            "Return plain JSON only. Do not use markdown, bullets, headings, or code fences."
        ),
        context_payload=json.dumps(prompt_context, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema_expectations=json.dumps(
            output_schema, ensure_ascii=True, indent=2, sort_keys=True
        ),
        output_schema=output_schema,
    )
