# Job Copilot Master Plan

## 1. Product
Job Copilot is a candidate operating system:
- profile understanding
- search intention building
- ingestion and normalization
- ranking and fit explanation
- action management
- learning loop from user behavior and outcomes

## 2. Product pillars
### A. Understanding the candidate
Input:
- CV/raw text
- profile preferences
- user actions over time
Output:
- canonical profile
- target roles
- seniority
- normalized skills
- search profile

### B. Understanding jobs
Input:
- job boards / scraped sources / feeds
Output:
- canonical job representation
- normalized company
- lifecycle state
- source metadata
- parsable filters

### C. Matching and ranking
Output:
- fit score
- explanation
- gaps
- confidence
- ranked results

### D. Action layer
User actions:
- save
- hide
- mark bad
- whitelist company
- blacklist company
- applied
- interviewing
- rejected
- offer

### E. Learning loop
Use feedback from:
- viewed jobs
- saved/hidden jobs
- good/bad labels
- whitelist/blacklist companies
- applications and outcomes

## 3. System design
### engine-api
Canonical domain + APIs + validation + persistence.

### ingestion
Source-specific scrape/fetch + normalization + dedupe + lifecycle.

### ml
Python sidecar for enrichment:
- richer extraction
- fit explanation
- reranking
- analytics and charts
- future LLM workflows

### web
Dashboard + profile + search intent + ranked jobs + action loops.

## 4. Near-term roadmap
1. Canonical role catalog
2. Search preferences / search profile
3. Source filtering
4. Ranking v2
5. Web refresh/query-state fixes
6. Job lists / company lists
7. Analytics endpoints + charts
8. LLM enrichment integration

## 5. Source filtering requirement
The user must be able to control where jobs come from:
- one or more sources enabled
- region-specific behavior
- source-level trust / quality later
- search must preserve source filters as structured fields

## 6. Lists to support
### Job-level
- saved
- hidden
- bad fit
- normal / undecided

### Company-level
- whitelist
- blacklist

These lists must influence ranking and UI presentation.

## 7. Unique algorithm direction
Use a hybrid engine:
- deterministic role/search/ranking baseline in Rust
- LLM enrichment in Python
- explicit merge/validation in Rust
- explanations and analytics generated from structured evidence, not only raw text

## 8. Charts and analytics
The system should eventually provide:
- application funnel charts
- source performance charts
- role distribution charts
- compensation trend charts
- fit-over-time charts
- company quality views
- activity timeline

## 9. Rule for LLM
LLM may suggest:
- richer skills
- fit explanations
- refined search terms
- company risk notes
- resume tailoring hints

LLM may not define canonical truth by itself.
