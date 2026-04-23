import json

from app.enrichment.cover_letter_draft.contract import (
    CoverLetterDraftPrompt,
    CoverLetterDraftRequest,
    cover_letter_draft_schema,
)


def build_cover_letter_draft_prompt(context: CoverLetterDraftRequest) -> CoverLetterDraftPrompt:
    prompt_context = context.model_dump(exclude={"profile_id"}, by_alias=True)
    output_schema = cover_letter_draft_schema()

    return CoverLetterDraftPrompt(
        system_instructions=(
            "You generate additive cover letter drafts for a job search copilot. "
            "Use only the provided deterministic context and optional additive enrichments. "
            "Do not change or reinterpret ranking, score, canonical IDs, source IDs, or entities. "
            "Do not invent employers, achievements, metrics, timelines, responsibilities, technologies, or experience that are not present in the provided context. "
            "Every claim in the draft must be supportable by the profile analysis, search profile, deterministic fit, ranked job payload, optional job-fit explanation, optional application coaching, optional feedback state, or raw profile text. "
            "If evidence is weak or missing, put it in evidence_gaps instead of filling it in. "
            "Keep the draft readable and concise. opening_paragraph and closing_paragraph must each be a single paragraph. body_paragraphs should contain one to three short paragraphs. "
            "key_claims_used must list the grounded claims actually used in the draft. tone_notes must describe how the draft is framed without adding new facts. "
            "Return plain JSON only. Do not use markdown, bullets, headings, salutations, signatures, or code fences."
        ),
        context_payload=json.dumps(prompt_context, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema_expectations=json.dumps(output_schema, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema=output_schema,
    )
