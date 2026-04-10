# Backlog

## Core architecture
- [ ] isolate legacy backend
- [ ] create Rust engine-api
- [ ] create Rust ingestion service
- [ ] create Python ML service
- [ ] introduce Postgres
- [ ] define contracts package

## Domain
- [ ] finalize entities
- [ ] define ranking input/output models
- [ ] define change event models
- [ ] define source adapter interfaces

## Ingestion
- [ ] raw snapshot model
- [ ] source fetch abstraction
- [ ] parser abstraction
- [ ] normalization pipeline
- [ ] dedupe pipeline
- [ ] refresh scheduler

## Search
- [ ] keyword search
- [ ] filters
- [ ] ranking formula v1
- [ ] ranking explanation
- [ ] recency score
- [ ] remote/seniority fit

## ML
- [ ] extraction endpoint
- [ ] fit analysis endpoint
- [ ] reranking endpoint
- [ ] structured outputs
- [ ] experiment dataset design

## Frontend
- [ ] search page cleanup
- [ ] job details page
- [ ] save job flow
- [ ] application status flow
- [ ] profile/preferences UI

## Quality
- [ ] unit tests for ranking
- [ ] integration tests for jobs/applications
- [ ] ingestion tests
- [ ] E2E smoke flow