export const queryKeys = {
  jobs: {
    all: () => ['jobs'] as const,
    filtered: (lifecycle: string, source: string | null, profileId?: string | null) =>
      ['jobs', 'filtered', lifecycle, source ?? 'all', profileId ?? 'none'] as const,
    detail: (id: string, profileId?: string | null) => ['jobs', id, profileId ?? 'none'] as const,
  },
  sources: {
    all: () => ['sources'] as const,
  },
  roles: {
    all: () => ['roles'] as const,
  },
  profile: {
    root: () => ['profile'] as const,
    rawText: () => ['profile', 'rawText'] as const,
    suggestedSkills: () => ['profile', 'suggestedSkills'] as const,
  },
  resumes: {
    all: () => ['resumes'] as const,
    active: () => ['resumes', 'active'] as const,
  },
  match: {
    forJob: (jobId: string) => ['match', jobId] as const,
  },
  applications: {
    all: () => ['applications'] as const,
    detail: (id: string) => ['applications', id] as const,
  },
  market: {
    insights: () => ['market', 'insights'] as const,
    overview: () => ['market', 'overview'] as const,
    companies: () => ['market', 'companies'] as const,
    salaries: () => ['market', 'salaries'] as const,
    roles: () => ['market', 'roles'] as const,
  },
  notifications: {
    all: () => ['notifications'] as const,
    list: (profileId: string, limit: number) =>
      ['notifications', 'list', profileId, limit] as const,
    unreadCount: (profileId: string) => ['notifications', 'unreadCount', profileId] as const,
  },
  alerts: {
    all: () => ['alerts'] as const,
  },
  dashboard: {
    stats: () => ['dashboard', 'stats'] as const,
  },
  contacts: {
    all: () => ['contacts'] as const,
  },
  activities: {
    forApp: (appId: string) => ['activities', appId] as const,
  },
  tasks: {
    forApp: (appId: string) => ['tasks', appId] as const,
    due: () => ['tasks', 'due'] as const,
  },
  coverLetters: {
    all: (jobId?: string) =>
      (jobId ? ['coverLetters', jobId] : ['coverLetters']) as readonly string[],
  },
  interviewQA: {
    all: (jobId?: string) =>
      (jobId ? ['interviewQA', jobId] : ['interviewQA']) as readonly string[],
  },
  offers: {
    all: () => ['offers'] as const,
  },
  search: {
    results: (q: string) => ['search', q] as const,
  },
  feedback: {
    profile: (profileId: string) => ['feedback', profileId] as const,
  },
  ml: {
    all: () => ['ml'] as const,
    ready: () => ['ml', 'ready'] as const,
    rerankPrefix: (profileId: string) => ['ml', 'rerank', profileId] as const,
    rerank: (profileId: string, jobsKey: string) => ['ml', 'rerank', profileId, jobsKey] as const,
    fitPrefix: (profileId: string) => ['ml', 'fit', profileId] as const,
    fit: (profileId: string, jobId: string) => ['ml', 'fit', profileId, jobId] as const,
    fitExplanationPrefix: (profileId: string) => ['ml', 'fitExplanation', profileId] as const,
    fitExplanation: (profileId: string, jobId: string) =>
      ['ml', 'fitExplanation', profileId, jobId] as const,
    coverLetterPrefix: (profileId: string) => ['ml', 'coverLetter', profileId] as const,
    coverLetter: (profileId: string, jobId: string) =>
      ['ml', 'coverLetter', profileId, jobId] as const,
    interviewPrepPrefix: (profileId: string) => ['ml', 'interviewPrep', profileId] as const,
    interviewPrep: (profileId: string, jobId: string) =>
      ['ml', 'interviewPrep', profileId, jobId] as const,
  },
  analytics: {
    all: () => ['analytics'] as const,
    summary: (profileId: string) => ['analytics', 'summary', profileId] as const,
    behavior: (profileId: string) => ['analytics', 'behavior', profileId] as const,
    funnel: (profileId: string) => ['analytics', 'funnel', profileId] as const,
    llmContext: (profileId: string) => ['analytics', 'llmContext', profileId] as const,
    rerankerMetrics: (profileId: string) => ['analytics', 'rerankerMetrics', profileId] as const,
    profileInsights: (profileId: string, contextVersion: string) =>
      ['analytics', 'profileInsights', profileId, contextVersion] as const,
    weeklyGuidance: (profileId: string, contextVersion: string) =>
      ['analytics', 'weeklyGuidance', profileId, contextVersion] as const,
    ingestionStats: () => ['analytics', 'ingestionStats'] as const,
  },
};
