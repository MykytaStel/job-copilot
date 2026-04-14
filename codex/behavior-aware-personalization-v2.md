Implement the next slice after event logging:

**behavior-aware personalization v2**

Context:
- profile events exist
- feedback persistence exists
- deterministic ranking exists
- feedback-aware personalization v1 exists
- analytics summary exists
- now we want search ranking to learn from repeated user behavior

Goal:
Use logged behavior to improve deterministic ranking in an explainable way.

Scope:
Primarily `apps/engine-api`.
No LLM changes in this task.
No major redesign.

What to implement:

1. Add a learning feature builder
Compute profile-level behavior features from:
- saved events
- hidden events
- bad_fit events
- company whitelist/blacklist
- source interactions

Need small explicit features such as:
- preferred_sources
- avoided_sources
- preferred_role_families
- avoided_role_families
- preferred_skill_terms
- avoided_terms if visible from repeated bad-fit/hide patterns

2. Add a behavior-aware ranking adjustment layer
This should remain deterministic and additive.

Examples:
- repeated positive interactions with a source => small boost
- repeated negative interactions with a source => small penalty
- repeated positive interactions with a role family => small boost
- repeated hide/bad_fit patterns for a role family => penalty
- company whitelist / blacklist rules stay as-is

3. Explanations
When learning signals affect ranking, add deterministic reasons such as:
- "Similar jobs from Djinni were frequently saved"
- "Jobs with similar role-family patterns were repeatedly hidden"
Keep reasons grounded and explicit.

4. Tests
Add tests proving:
- repeated saves can improve ranking
- repeated hides/bad-fit can reduce ranking
- ranking remains deterministic
- explanations include learning-signal reasons

Constraints:
- do not replace the base deterministic scorer
- no LLM in this task
- keep logic explicit and tunable

Acceptance criteria:
- repeated user behavior can affect ranking
- behavior-aware deltas are explainable
- base deterministic engine remains intact
