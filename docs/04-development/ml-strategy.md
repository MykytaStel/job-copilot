# ML Strategy — без платного AI API

> Стратегія: максимальна цінність без per-call витрат.
> Paid AI API підключається тільки у платній підписці.

## Рівні ML в системі

### Рівень 1 — Детермінований scoring (Rust, завжди активний)
**Файл:** `apps/engine-api/src/services/matching.rs`

Основний двигун. Ніяких API викликів. Ніяких залежностей.

Поточні ваги:
- Primary role: 22.0
- Profile skills: 20.0
- Target roles: 12.0
- Role candidates: 10.0
- Profile keywords: 8.0
- Search terms: 8.0

Бонуси/штрафи: source, work mode, region, seniority, exclude terms, role mismatch.

**Що додати (Фаза 1):**
- Freshness decay: `score * (1.0 - days_old / 30.0).max(0.7)` для jobs > 14 днів
- Company reputation: whitelist +5, blacklist -20
- Salary fit: ±8 якщо профіль має salary range і job має salary data
- Description penalty: -5 якщо description < 200 chars

### Рівень 2 — Поведінковий scoring (Rust, активний)
**Файл:** `apps/engine-api/src/services/behavior.rs`

Агрегує user signals:
- Per-source: +2/+4 boost за позитивні, -2/-4 за негативні
- Per-role-family: +1/+3 по save/hide/badfit history
- Saturation: `count / SATURATION`, max 1.0

### Рівень 3 — Trained reranker (Logistic Regression, Python)
**Файл:** `apps/ml/app/trained_reranker.py`

**Проблема зараз:** 4 labeled examples → ваги ~0.

**Рішення — Bootstrap pipeline:**

```python
# apps/ml/app/bootstrap_training.py
# Працює з export dataset від engine-api, а не з сирим event→label маппінгом.
# export already uses label_policy_version = outcome_label_v2

example["signals"] = {
    "viewed": True,
    "saved": True,
    "hidden": False,
    "bad_fit": False,
    "applied": False,
    "dismissed": False,
    "explicit_feedback": True,
    "explicit_saved": True,
    "explicit_hidden": False,
    "explicit_bad_fit": False,
    "viewed_event_count": 1,
    "saved_event_count": 1,
    "applied_event_count": 0,
    "dismissed_event_count": 0,
}
# label_reasons: ["saved"] / ["viewed"] / ["applied"] / ["dismissed", ...]
# Далі передає готові normalized examples в trained_reranker.train()
```

**Коли retrain:**
- Порогова умова: ≥ 30 labelable outcome examples після v2 normalization
- Тригер: ручний виклик `POST /api/v1/reranker/retrain` або CLI
- Зберігати нову модель в `models/trained-reranker-v2.json`

**Реалістична мета:** після 2-3 тижнів активного використання, 50+ feedbacks.

### Рівень 4 — Template enrichment (Python, активний зараз)
**Файл:** `apps/ml/app/llm_provider.py` → `TemplateEnrichmentProvider`

Генерує структуровані тексти без LLM на основі:
- matched_terms, missing_terms, score, reasons
- profile summary, skills
- job title, company, description

**Що покращити (без LLM):**
- Profile insights: "You match X% of required skills. Strong areas: {top_skills}. Missing: {gaps}."
- Weekly guidance: аналіз funnel + behavior signals → actionable text
- Job fit explanation: зв'язний текст із matched/missing/reasons структур

### Рівень 5 — Ollama (self-hosted, Фаза 4)
**Файл:** `apps/ml/app/llm_provider.py` → `OllamaEnrichmentProvider` (новий)

```python
class OllamaEnrichmentProvider:
    def __init__(self, base_url: str, model: str = "mistral:7b"):
        self.base_url = base_url  # OLLAMA_BASE_URL env
        self.model = model        # OLLAMA_MODEL env

    async def call(self, prompt: str) -> str:
        # POST {base_url}/api/generate
        ...
```

**Моделі для розгляду:**
- `mistral:7b` — хороший баланс якість/розмір
- `qwen2.5:3b` — менший, швидший, гірший
- `llama3.1:8b` — кращий але важчий

**Вартість:** $10-20/міс VPS (замість per-call витрат)

**Env vars:**
```
ML_LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://ollama:11434
OLLAMA_MODEL=mistral:7b
```

### Рівень 6 — Real LLM API (тільки paid tier, Фаза 5)

```
ML_LLM_PROVIDER=anthropic   → AnthropicEnrichmentProvider (claude-haiku-4-5)
ML_LLM_PROVIDER=openai      → OpenAIEnrichmentProvider (gpt-4o-mini)
ML_LLM_PROVIDER=ollama      → OllamaEnrichmentProvider
ML_LLM_PROVIDER=template    → TemplateEnrichmentProvider (default/free)
```

## Provider switching logic

```python
def get_enrichment_provider() -> BaseEnrichmentProvider:
    provider = os.getenv("ML_LLM_PROVIDER", "template")
    if provider == "anthropic":
        return AnthropicEnrichmentProvider(api_key=os.getenv("ANTHROPIC_API_KEY"))
    elif provider == "openai":
        return OpenAIEnrichmentProvider(api_key=os.getenv("OPENAI_API_KEY"))
    elif provider == "ollama":
        return OllamaEnrichmentProvider(base_url=os.getenv("OLLAMA_BASE_URL"))
    return TemplateEnrichmentProvider()
```

## Що НЕ робимо зараз

- Не додаємо PyTorch / TensorFlow / scikit-learn (занадто важко для 4 прикладів)
- Не використовуємо semantic embeddings поки немає 100+ labeled pairs
- Не платимо за API поки є template fallback + Ollama plan
- Не ускладнюємо модель: logistic regression достатньо для цього масштабу
