export type CvTailoringGapItem = {
  skill: string;
  suggestion: string;
};

export type CvTailoringSuggestions = {
  skillsToHighlight: string[];
  skillsToMention: string[];
  gapsToAddress: CvTailoringGapItem[];
  summaryRewrite: string;
  keyPhrases: string[];
};

export type CvTailoringResponse = {
  suggestions: CvTailoringSuggestions;
  provider: string;
  generatedAt: string;
};