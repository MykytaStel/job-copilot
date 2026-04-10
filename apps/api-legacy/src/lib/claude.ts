import Anthropic from '@anthropic-ai/sdk';

// Fail fast at startup if key is missing
if (!process.env.ANTHROPIC_API_KEY) {
  throw new Error('ANTHROPIC_API_KEY is not set. Create apps/api-legacy/.env from .env.example.');
}

const client = new Anthropic({ apiKey: process.env.ANTHROPIC_API_KEY });

const MODEL = 'claude-sonnet-4-6';

/**
 * Send a single user prompt and return the text response.
 * Throws if the API is unavailable or returns a non-text block.
 */
export async function callClaude(prompt: string, maxTokens = 1024): Promise<string> {
  const message = await client.messages.create({
    model: MODEL,
    max_tokens: maxTokens,
    messages: [{ role: 'user', content: prompt }],
  });

  const block = message.content[0];
  if (!block || block.type !== 'text') {
    throw new Error('Claude returned an unexpected response format');
  }
  return block.text;
}

/**
 * Like callClaude but parses the response as JSON.
 * Throws with a clear message if parsing fails.
 */
export async function callClaudeJson<T>(prompt: string, maxTokens = 1024): Promise<T> {
  const text = await callClaude(prompt, maxTokens);
  // Strip markdown code fences if Claude wraps the JSON
  const cleaned = text.replace(/^```(?:json)?\s*/i, '').replace(/\s*```$/, '').trim();
  try {
    return JSON.parse(cleaned) as T;
  } catch {
    throw new Error(`Claude response was not valid JSON: ${cleaned.slice(0, 200)}`);
  }
}
