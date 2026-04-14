# Target Architecture

## Domain-first architecture
- role taxonomy
- profile/search profile
- job / job variant / company
- fit score / explanation
- lists / actions / outcomes

## LLM integration pattern
1. deterministic Rust baseline
2. Python enrichment
3. validated merge in Rust

## Search pipeline
candidate -> analyzed profile -> search profile -> source-filtered search -> rank -> explain -> action
