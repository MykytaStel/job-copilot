# Claude prompt — Learning Layer Architecture

Use the current repository, docs, and implemented slices as the source of truth.

Task:
Design the next architecture slice for:

**event logging + learning signals v1**

Context:
- deterministic profile analysis exists
- search-profile build exists
- deterministic ranked search exists
- feedback-aware personalization exists
- analytics summary exists
- LLM enrichments exist:
  - profile insights
  - job-fit explanation
  - application coaching
  - cover letter draft
- we now want the system to actually learn from user behavior over time

What to do:
1. Assess what behavioral signals already exist in the project.
2. Propose the smallest stable architecture for:
   - event logging
   - learning signals
   - profile behavior memory
   - chart/analytics reuse
3. Keep Rust as source of truth.
4. Keep LLM as additive layer.
5. Do not propose a giant rewrite.

Output format:
1. Current gaps
2. Target learning architecture
3. Event model
4. Backend/domain design
5. Analytics/aggregate design
6. Minimal first slice
7. File-by-file plan
8. Codex-ready implementation brief

Important:
- prefer stable domain contracts over clever abstractions
- do not jump to fine-tuning first
- focus on event logging and learning signals
- be concrete
