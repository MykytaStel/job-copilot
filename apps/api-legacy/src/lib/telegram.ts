const BOT_TOKEN = process.env.TELEGRAM_BOT_TOKEN;

/**
 * Sends a message via Telegram Bot API.
 * Silently no-ops if TELEGRAM_BOT_TOKEN is not set.
 */
export async function sendTelegramMessage(chatId: string, text: string): Promise<void> {
  if (!BOT_TOKEN) return;
  const url = `https://api.telegram.org/bot${BOT_TOKEN}/sendMessage`;
  const res = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ chat_id: chatId, text, parse_mode: 'HTML' }),
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(`Telegram error: ${JSON.stringify(body)}`);
  }
}
