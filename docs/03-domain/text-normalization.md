# Text Normalization

This contract defines pre-match normalization for text used by `engine-api` and `ml`.

It is an internal matching rule, not a domain-contract change:
- canonical `RoleId` values stay unchanged
- DTO shapes stay unchanged
- API-facing matched terms remain human-readable

## Pipeline

1. lowercase input
2. rewrite symbol-heavy aliases before punctuation stripping
3. replace remaining punctuation with whitespace
4. collapse known compound terms and multi-word signals into phrase-safe atoms
5. tokenize on whitespace after phrase folding

## Canonical compound forms

These must normalize before tokenization:
- `front-end`, `front end`, `frontend` -> `frontend`
- `back-end`, `back end`, `backend` -> `backend`
- `full-stack`, `full stack`, `fullstack` -> `fullstack`
- `react native`, `react-native`, `reactnative` -> `react_native`
- `node.js`, `node js`, `nodejs` -> `nodejs`
- `c#`, `c sharp` -> `csharp`
- `c++`, `c plus plus` -> `cpp`

## Phrase-safe multi-word signals

Known multi-word signals should be folded into underscore atoms before tokenization so they do not degrade into generic fragments:
- `react_native`
- `distributed_systems`
- `design_system`
- `product_company`
- `google_ads`
- `power_bi`
- `customer_support`
- `quality_assurance`
- `test_automation`
- `data_analyst`
- `product_manager`
- `product_management`
- `project_manager`
- `project_management`
- `social_media`
- `lead_generation`
- `candidate_screening`
- `talent_acquisition`
- `help_desk`

## Output rule

Internal matching may use phrase-safe atoms such as `react_native`.
User-facing evidence and matched term lists should render those back as human-readable phrases such as `react native`.
