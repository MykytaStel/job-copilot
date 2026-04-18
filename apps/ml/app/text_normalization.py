import re


NON_ALNUM_RE = re.compile(r"[^a-z0-9]+")

RAW_ALIAS_REPLACEMENTS: tuple[tuple[str, str], ...] = (
    ("c++", " cpp "),
    ("c#", " csharp "),
    ("node.js", " nodejs "),
    ("react.js", " react "),
    ("reactnative", " react native "),
)

PHRASE_REWRITES: tuple[tuple[tuple[str, ...], str], ...] = (
    (("c", "plus", "plus"), "cpp"),
    (("candidate", "screening"), "candidate_screening"),
    (("distributed", "systems"), "distributed_systems"),
    (("google", "ads"), "google_ads"),
    (("lead", "generation"), "lead_generation"),
    (("product", "management"), "product_management"),
    (("project", "management"), "project_management"),
    (("quality", "assurance"), "quality_assurance"),
    (("social", "media"), "social_media"),
    (("talent", "acquisition"), "talent_acquisition"),
    (("test", "automation"), "test_automation"),
    (("customer", "support"), "customer_support"),
    (("data", "analyst"), "data_analyst"),
    (("design", "system"), "design_system"),
    (("front", "end"), "frontend"),
    (("back", "end"), "backend"),
    (("full", "stack"), "fullstack"),
    (("help", "desk"), "help_desk"),
    (("node", "js"), "nodejs"),
    (("power", "bi"), "power_bi"),
    (("product", "company"), "product_company"),
    (("product", "manager"), "product_manager"),
    (("project", "manager"), "project_manager"),
    (("react", "native"), "react_native"),
    (("c", "sharp"), "csharp"),
)

STOPWORDS = {
    "a",
    "an",
    "and",
    "are",
    "as",
    "at",
    "be",
    "by",
    "for",
    "from",
    "in",
    "into",
    "is",
    "of",
    "on",
    "or",
    "the",
    "to",
    "with",
}

LOW_SIGNAL_TERMS = {
    "developer",
    "engineer",
    "specialist",
    "manager",
    "experience",
    "experienced",
    "role",
    "roles",
    "work",
    "working",
    "team",
    "teams",
}


def normalize_text(value: str) -> str:
    normalized = value.lower()
    for needle, replacement in RAW_ALIAS_REPLACEMENTS:
        normalized = normalized.replace(needle, replacement)

    collapsed = NON_ALNUM_RE.sub(" ", normalized).strip()
    if not collapsed:
        return ""

    tokens = collapsed.split()
    return " ".join(_rewrite_known_phrases(tokens))


def normalize_term_for_output(value: str) -> str:
    return normalize_text(value).replace("_", " ")


def extract_terms(value: str) -> list[str]:
    normalized = normalize_text(value)
    if not normalized:
        return []

    terms: list[str] = []
    for token in normalized.split():
        display_value = token.replace("_", " ")

        if "_" in token:
            terms.append(display_value)
            continue

        if len(token) < 2 or token in STOPWORDS or token in LOW_SIGNAL_TERMS:
            continue

        terms.append(display_value)

    return terms


def tokenize(*chunks: str | None) -> list[str]:
    tokens: list[str] = []
    for chunk in chunks:
        if not chunk:
            continue
        tokens.extend(extract_terms(chunk))
    return tokens


def _rewrite_known_phrases(tokens: list[str]) -> list[str]:
    rewritten: list[str] = []
    index = 0

    while index < len(tokens):
        matched = False

        for pattern, replacement in PHRASE_REWRITES:
            size = len(pattern)
            if tuple(tokens[index : index + size]) != pattern:
                continue

            rewritten.append(replacement)
            index += size
            matched = True
            break

        if matched:
            continue

        rewritten.append(tokens[index])
        index += 1

    return rewritten
