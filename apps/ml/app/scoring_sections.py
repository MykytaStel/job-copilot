from __future__ import annotations

from dataclasses import dataclass

from app.text_normalization import normalize_text

REQUIRED_SECTION_WEIGHT = 1.6
PREFERRED_SECTION_WEIGHT = 0.7
EARLY_OR_REPEATED_TERM_WEIGHT = 1.25

REQUIRED_SECTION_MARKERS = {
    "requirements",
    "required",
    "must have",
    "mandatory",
    "qualifications",
    "essential",
    "required skills",
    "must have skills",
    "minimum qualifications",
    "key requirements",
    "technical requirements",
    "minimum requirements",
    "what we require",
    "what we need",
    "you must have",
    "you should have",
    "you will need",
    "hard requirements",
    "skills required",
    "experience required",
}

PREFERRED_SECTION_MARKERS = {
    "preferred",
    "nice to have",
    "nice to haves",
    "bonus",
    "desirable",
    "good to have",
    "preferred skills",
    "preferred qualifications",
    "bonus skills",
    "bonus points",
    "would be a plus",
    "optional",
    "it would be great",
}


@dataclass(frozen=True)
class SkillSections:
    required_text: str
    preferred_text: str
    full_text: str

    @property
    def has_explicit_sections(self) -> bool:
        return bool(self.required_text or self.preferred_text)


def parse_skill_sections(description_text: str) -> SkillSections:
    required_lines: list[str] = []
    preferred_lines: list[str] = []
    current: str | None = None

    for line in description_text.splitlines():
        trimmed = line.strip()

        if not trimmed:
            continue

        clean = _clean_section_header(trimmed)

        if len(trimmed) <= 80 and clean in REQUIRED_SECTION_MARKERS:
            current = "required"
            continue

        if len(trimmed) <= 80 and clean in PREFERRED_SECTION_MARKERS:
            current = "preferred"
            continue

        if current == "required":
            required_lines.append(trimmed)
        elif current == "preferred":
            preferred_lines.append(trimmed)

    return SkillSections(
        required_text=normalize_text(" ".join(required_lines)),
        preferred_text=normalize_text(" ".join(preferred_lines)),
        full_text=normalize_text(description_text),
    )


def section_weight_for_term(term_key_value: str, sections: SkillSections) -> float:
    if sections.required_text and _term_matches_text(term_key_value, sections.required_text):
        return REQUIRED_SECTION_WEIGHT

    if sections.preferred_text and _term_matches_text(term_key_value, sections.preferred_text):
        return PREFERRED_SECTION_WEIGHT

    if not sections.has_explicit_sections:
        frequency = sections.full_text.count(term_key_value)
        position = sections.full_text.find(term_key_value)

        if frequency >= 2 or (0 <= position <= 300):
            return EARLY_OR_REPEATED_TERM_WEIGHT

    return 1.0


def _clean_section_header(line: str) -> str:
    return " ".join(
        "".join(
            char.lower() if char.isalpha() or char.isspace() else " "
            for char in line.replace("-", " ")
        ).split()
    )


def _term_matches_text(term_key_value: str, text: str) -> bool:
    if not term_key_value or not text:
        return False

    if term_key_value in text:
        return True

    parts = term_key_value.split()

    return bool(parts) and all(part in text for part in parts)
