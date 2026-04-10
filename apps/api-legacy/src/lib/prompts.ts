const MAX_JOB = 4000;
const MAX_RESUME = 6000;

export function fitScoringPrompt(jobDescription: string, resumeText: string): string {
  return `You are a senior technical recruiter. Analyze how well this candidate fits the job.

<job_description>
${jobDescription.slice(0, MAX_JOB)}
</job_description>

<candidate_resume>
${resumeText.slice(0, MAX_RESUME)}
</candidate_resume>

Return ONLY valid JSON — no markdown, no explanation, just the object:
{
  "score": <integer 0-100 reflecting genuine fit>,
  "matchedSkills": [<specific skills/technologies present in both job and resume>],
  "missingSkills": [<specific requirements from job that are absent from resume>],
  "notes": "<2-3 honest sentences: overall fit, biggest gap, strongest point>"
}`;
}
