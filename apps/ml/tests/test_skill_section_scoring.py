from app.scoring import parse_skill_sections, section_weight_for_term


def test_required_skill_weight_is_higher_than_preferred_weight() -> None:
    sections = parse_skill_sections(
        """
        Requirements:
        - Rust
        - PostgreSQL

        Nice to have:
        - GraphQL
        """
    )

    assert section_weight_for_term("rust", sections) > section_weight_for_term("graphql", sections)


def test_fallback_heuristic_boosts_early_or_repeated_terms() -> None:
    sections = parse_skill_sections(
        "Rust backend role. We use Rust daily. "
        + ("general backend product engineering text " * 12)
        + "GraphQL appears only near the end."
    )

    assert section_weight_for_term("rust", sections) > section_weight_for_term("graphql", sections)
