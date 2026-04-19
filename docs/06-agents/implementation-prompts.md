# Implementation Prompts

Кожен промпт — окремий чат. Копіюй як є, він самодостатній.

---

## ФАЗА 1 — Основи

---

### 1.1 — Sidebar: реальний профіль

```
Job Copilot monorepo (Rust engine-api + Python ml + React web).

Задача: у файлі `apps/web/src/AppShellNew.tsx` близько рядка 253 є hardcoded текст замість реального profile name/email. Треба підключити реальний профіль.

Контекст:
- Profile ID зберігається в `window.localStorage.getItem('engine_api_profile_id')`
- Є React Query hook — `useQuery` для GET /api/v1/profiles/:id, його вже використовує useProfilePage.ts
- Profile має поля: name, email, location
- Sidebar має показувати name + email, якщо профіль не завантажений — показувати скелетон або "—"

Що зробити:
1. Прочитай `apps/web/src/AppShellNew.tsx` і знайди hardcoded місце
2. Прочитай `apps/web/src/features/profile/useProfilePage.ts` щоб зрозуміти як робиться запит до профілю
3. Додай мінімальний profile query в AppShellNew (або винеси в маленький хук) і підстав реальні дані у sidebar
4. Не дублюй логіку — якщо є спільний тип Profile, використай його

Перевірка: sidebar показує реальне ім'я після того як профіль збережений.
```

---

### 1.2 — Query invalidation після feedback

```
Job Copilot monorepo (Rust engine-api + Python ml + React web).

Задача: після дій save/hide/badFit у Dashboard — ML rerank результати не оновлюються бо staleTime = 5 хв. Треба примусово інвалідувати.

Контекст:
- Файл: `apps/web/src/pages/Dashboard.tsx`
- Query keys визначені в `apps/web/src/queryKeys.ts`
- Після mutation onSuccess вже інвалідується: jobs.all(), applications.all(), dashboard.stats(), feedback.profile(profileId)
- НЕ інвалідується: ml.rerank ключі
- Після зміни профілю (useProfilePage.ts) — не інвалідуються ml.rerank та analytics ключі

Що зробити:
1. Прочитай `apps/web/src/pages/Dashboard.tsx` — знайди всі mutation onSuccess блоки (save, hide, badFit, unmark)
2. Прочитай `apps/web/src/queryKeys.ts` — знайди ml.rerank ключ
3. В кожному onSuccess після feedback мутацій — додай invalidateQueries для ml.rerank(profileId)
4. Прочитай `apps/web/src/features/profile/useProfilePage.ts` — знайди saveMutation і analyzeMutation
5. В їх onSuccess — додай invalidateQueries для ml.rerank та analytics ключів

Перевірка: після hide або save job — ranked list оновлюється без чекання 5 хвилин.
```

---

### 1.3 — Canonical role catalog

```
Job Copilot monorepo (Rust engine-api + ingestion + React web).

Задача: створити canonical role catalog в engine-api і підключити до scoring та Profile UI.

Контекст:
- Файл scoring: `apps/engine-api/src/services/matching.rs` — PRIMARY_ROLE_WEIGHT=22.0, зараз порівнює free-form strings
- Profile Builder у `apps/web/src/pages/Profile.tsx` вже має multi-select для ролей
- Є endpoint або константи для ролей — перевір `apps/engine-api/src/`
- CLAUDE.md: "Internal role identity must use a canonical role model (RoleId / role catalog), not free-form strings"

Ролі для каталогу (мінімальний набір для UA ринку):
frontend_engineer, backend_engineer, fullstack_engineer, mobile_engineer,
devops_engineer, data_engineer, ml_engineer, qa_engineer,
product_designer, product_manager, project_manager, tech_lead, engineering_manager

Що зробити:
1. Прочитай `apps/engine-api/src/` структуру — знайди де domain моделі
2. Створи `apps/engine-api/src/domain/role/catalog.rs`:
   - RoleId enum з усіма ролями
   - display_name() метод
   - aliases: Vec<&str> для кожної ролі (наприклад frontend_engineer → ["frontend", "react developer", "vue developer", "angular developer"])
   - from_str() що матчить aliases
3. Додай endpoint `GET /api/v1/roles` що повертає список {id, display_name}
4. Прочитай `apps/web/src/features/profile/useProfilePage.ts` — знайди де завантажуються ролі для multi-select
5. Підключи новий endpoint до Profile UI

Перевірка: GET /api/v1/roles повертає список ролей; Profile Builder показує ролі з каталогу.
```

---

### 1.4 — Виправити LLM provider + підготувати Ollama path

```
Job Copilot monorepo. ML сервіс на Python FastAPI.

Задача: виправити llm_provider.py — поточний OpenAI model name `gpt-5.4-mini` не існує. Також підготувати OllamaEnrichmentProvider.

Контекст:
- Файл: `apps/ml/app/llm_provider.py`
- Є TemplateEnrichmentProvider (шаблонний, працює без API) — залишити як default
- Є OpenAIEnrichmentProvider — використовує `gpt-5.4-mini` (неправильно)
- Provider вибирається через env var `ML_LLM_PROVIDER`
- Файл: `infra/docker-compose.yml` — там env vars для ml сервісу

Що зробити:
1. Прочитай `apps/ml/app/llm_provider.py` повністю
2. Виправ model name: `gpt-5.4-mini` → `gpt-4o-mini` (або читати з env `OPENAI_MODEL`, default `gpt-4o-mini`)
3. Створи `OllamaEnrichmentProvider` клас:
   - читає `OLLAMA_BASE_URL` (default: http://localhost:11434) та `OLLAMA_MODEL` (default: mistral:7b)
   - POST {base_url}/api/generate з {model, prompt, stream: false}
   - повертає response.json()["response"]
   - реалізує ті самі методи що OpenAIEnrichmentProvider
4. У функції `get_enrichment_provider()`:
   - `ML_LLM_PROVIDER=ollama` → OllamaEnrichmentProvider
   - `ML_LLM_PROVIDER=openai` → OpenAIEnrichmentProvider
   - default → TemplateEnrichmentProvider
5. Додай в `infra/docker-compose.yml` (ml сервіс) env vars: OLLAMA_BASE_URL, OLLAMA_MODEL (обидва optional/закоментовані)

Перевірка: TemplateEnrichmentProvider залишається default; при ML_LLM_PROVIDER=ollama — використовується Ollama endpoint.
```

---

### 1.5 — Bootstrap training data для ML reranker

```
Job Copilot monorepo. ML сервіс на Python FastAPI.

Задача: reranker натренований на 4 прикладах (ваги ~0, модель не працює). Треба написати скрипт що генерує labeled examples з реальних user events.

Контекст:
- Файл: `apps/ml/app/trained_reranker.py` — там є train() pipeline і структура labeled examples
- Файл: `apps/ml/models/trained-reranker-v2.json` — поточна (нефункціональна) модель
- Engine-API має endpoint для export dataset — перевір `apps/engine-api/src/api/` (шукай reranker_dataset)
- Файл: `apps/ml/app/engine_api_client.py` — клієнт до engine-api

Маппінг подій до labels:
- event_type = 'job_saved' → label = 1 (позитивний)
- event_type = 'job_applied' → label = 1 (позитивний)
- event_type = 'job_hidden' → label = 0 (негативний)
- event_type = 'job_bad_fit' → label = 0 (негативний)

Що зробити:
1. Прочитай `apps/ml/app/trained_reranker.py` — зрозумій формат labeled examples (які features потрібні)
2. Прочитай `apps/ml/app/engine_api_client.py` — зрозумій як спілкуватися з engine-api
3. Знайди endpoint для export reranker dataset в engine-api
4. Створи `apps/ml/app/bootstrap_training.py`:
   - Функція `fetch_labeled_examples(profile_id, engine_api_base_url)` → дістає dataset з engine-api
   - Функція `bootstrap_and_retrain(profile_id, min_examples=30)`:
     - якщо examples < min_examples → логує попередження і виходить
     - інакше → запускає retrain і зберігає нову модель
   - CLI: `python bootstrap_training.py --profile-id <id> --min-examples 30`
5. Додай endpoint `POST /api/v1/reranker/bootstrap` в ML сервіс що викликає bootstrap_and_retrain

Перевірка: скрипт запускається, якщо є 30+ feedbacks — ретрейнить і зберігає нову модель.
```

---

### 1.6 — Freshness decay + salary fit у scoring

```
Job Copilot monorepo. Engine-API на Rust.

Задача: додати freshness decay і salary fit до scoring алгоритму.

Контекст:
- Файл: `apps/engine-api/src/services/matching.rs`
- Метод `score_job` (приблизно рядки 152-269) — там є основний scoring
- Job має поля: `last_seen_at` (DateTime), `salary_min`, `salary_max` (Option<i32>), `salary_currency`
- Profile має `search_profile` з можливими полями salary_min/salary_max — перевір чи є вони
- Score clamped 0-100 в кінці

Що зробити:
1. Прочитай `apps/engine-api/src/services/matching.rs` — знайди score_job метод
2. Додай freshness decay ПІСЛЯ основного розрахунку score (перед clamp):
   ```rust
   let days_old = (now - job.last_seen_at).num_days().max(0) as f32;
   if days_old > 14.0 {
       let decay = (1.0 - (days_old - 14.0) / 30.0).max(0.7);
       score *= decay;
   }
   ```
3. Перевір Profile/SearchProfile — чи є salary_min/salary_max поля
   - Якщо є: додай salary fit бонус/штраф:
     - job.salary_min і profile.salary_min обидва є → порівняй діапазони
     - Перетин діапазонів → +6 бонус
     - Job salary значно нижче від profile min → -8 штраф
   - Якщо немає salary у Profile — пропусти цей блок
4. Додай company reputation:
   - Якщо компанія є в whitelist → +5
   - Якщо в blacklist → -20 (можна розглянути як filter, або сильний penalty)
   - Перевір де зберігається whitelist/blacklist в domain моделі
5. Додай explanation рядки до reasons array для кожного нового сигналу

Перевірка: jobs > 14 днів мають нижчий score; whitelist companies мають вищий; rust tests компілюються.
```

---

## ФАЗА 2 — Market Intelligence

---

### 2.1 — market_snapshots міграція + company stats endpoint

```
Job Copilot monorepo. Engine-API на Rust + PostgreSQL.

Задача: створити таблицю market_snapshots і перші endpoints для market intelligence.

Контекст:
- Міграції: `apps/engine-api/migrations/` — формат файлів: `{timestamp}_{name}.sql`
- Останні міграції — перевір timestamp щоб дати правильний наступний
- Engine-API роути: `apps/engine-api/src/api/` — там є routes, handlers, dto
- Є JobRepository — перевір `apps/engine-api/src/db/repositories/`

Що зробити:
1. Прочитай структуру `apps/engine-api/migrations/` — знайди останній timestamp
2. Створи міграцію `{next_timestamp}_add_market_snapshots.sql`:
   ```sql
   CREATE TABLE market_snapshots (
     id TEXT PRIMARY KEY,
     snapshot_date DATE NOT NULL,
     snapshot_type TEXT NOT NULL,
     payload JSONB NOT NULL,
     created_at TIMESTAMPTZ DEFAULT NOW()
   );
   CREATE INDEX ON market_snapshots(snapshot_date, snapshot_type);
   CREATE INDEX ON jobs USING GIN(to_tsvector('simple', title || ' ' || description_text));
   ```
3. Прочитай `apps/engine-api/src/api/` — зрозумій pattern для нових routes
4. Створи `apps/engine-api/src/api/market.rs` з handlers:
   - `GET /api/v1/market/overview` — повертає: new_jobs_this_week, active_companies_count, active_jobs_count, remote_percentage
   - `GET /api/v1/market/companies?limit=20` — топ компаній по кількості активних вакансій + velocity (this_week vs prev_week)
5. Додай відповідні DTOs і підключи routes
6. Spec детально в `docs/03-domain/market-intelligence.md`

Перевірка: GET /api/v1/market/overview повертає реальні дані; GET /api/v1/market/companies повертає список.
```

---

### 2.2 — Salary trends + role demand endpoints

```
Job Copilot monorepo. Engine-API на Rust + PostgreSQL.

Задача: додати salary trends і role demand endpoints до market intelligence API.

Контекст:
- Попередній task (2.1) вже створив `market.rs` з overview і companies endpoints
- Файл: `apps/engine-api/src/api/market.rs`
- Таблиця jobs має: salary_min, salary_max (Option<i32>), salary_currency, seniority, title, first_seen_at
- Spec: `docs/03-domain/market-intelligence.md`

Що зробити:
1. Прочитай `apps/engine-api/src/api/market.rs`
2. Додай endpoint `GET /api/v1/market/salaries?seniority=senior`:
   - Повертає: {seniority, p25, median, p75, sample_count} для jobs з salary data за останні 30 днів
   - SQL: PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY salary_min)
   - Групуй по seniority
3. Додай endpoint `GET /api/v1/market/roles?period=30`:
   - Порахуй кількість нових вакансій по групах ролей за вказаний period і попередній такий самий period
   - Групи: Frontend, Backend, Fullstack, DevOps, Data/ML, QA, Design, Management
   - Матч по title keywords (або RoleId catalog якщо вже є)
   - Повертає: [{role_group, this_period, prev_period, trend: "up"|"down"|"stable"}]
4. Додай DTOs для обох
5. Підключи до router

Перевірка: обидва endpoints повертають реальні дані; sample_count > 0 якщо є вакансії з salary.
```

---

### 2.3 — Market Intelligence сторінка у Web

```
Job Copilot monorepo. React 19 + TanStack Query web app.

Задача: створити сторінку /market з ринковою аналітикою.

Контекст:
- Роути: `apps/web/src/App.tsx` — там всі route визначення
- API клієнт: `apps/web/src/api.ts` — там всі функції для запитів
- Query keys: `apps/web/src/queryKeys.ts`
- Стиль: dark theme, Tailwind CSS, компоненти з `apps/web/src/components/ui/`
- Є StatCard компонент — перевір components/ui/
- Endpoints вже є: GET /api/v1/market/overview, /companies, /salaries, /roles

Що зробити:
1. Прочитай `apps/web/src/api.ts` — зрозумій pattern для нових API функцій
2. Додай в api.ts: getMarketOverview(), getMarketCompanies(), getMarketSalaries(), getMarketRoles()
3. Додай в queryKeys.ts: market.overview(), market.companies(), market.salaries(), market.roles()
4. Створи `apps/web/src/pages/Market.tsx`:
   - Секція 1: 4 stat cards (нові вакансії цього тижня, активних компаній, медіана зарплати, % remote)
   - Секція 2: "Top Hiring Companies" — table або list з company + jobs count + trend arrow
   - Секція 3: "Salary by Seniority" — simple bar або range display (p25-p75 range, median dot)
   - Секція 4: "Role Demand" — список ролей з trend up/down indicators
   - Loading states та empty states для кожної секції
5. Додай route /market в App.tsx
6. Додай "Market" пункт у навігацію в AppShellNew.tsx (після Analytics)

UX: quiet, dark, dense. Використовуй існуючі компоненти максимально.

Перевірка: /market відкривається, показує реальні дані; навігація веде на сторінку.
```

---

## ФАЗА 3 — User Engagement

---

### 3.1 — Notifications: міграція + API + UI

```
Job Copilot monorepo. Rust engine-api + React web.

Задача: повна реалізація in-app notifications — таблиця, API, UI сторінка.

Контекст:
- Bell icon вже є в AppShellNew.tsx але disabled/без route
- Notification types: new_jobs_found, job_reactivated, application_due_soon
- Тригер: при ingestion run — перевірити нові jobs для профілів і створити notifications
- Spec: `docs/05-roadmap/priority-roadmap.md` (Фаза 3.1)

Що зробити:
1. Створи міграцію в `apps/engine-api/migrations/`:
   ```sql
   CREATE TABLE notifications (
     id TEXT PRIMARY KEY,
     profile_id TEXT REFERENCES profiles(id) ON DELETE CASCADE,
     type TEXT NOT NULL,
     title TEXT NOT NULL,
     body TEXT,
     payload JSONB,
     read_at TIMESTAMPTZ,
     created_at TIMESTAMPTZ DEFAULT NOW()
   );
   CREATE INDEX ON notifications(profile_id, read_at);
   CREATE INDEX ON notifications(profile_id, created_at DESC);
   ```
2. Прочитай `apps/engine-api/src/api/` — зрозумій pattern
3. Створи notifications router з:
   - `GET /api/v1/notifications?profile_id=&limit=20` — список, новіші першими
   - `POST /api/v1/notifications/:id/read` — позначити прочитаним
   - `GET /api/v1/notifications/unread-count?profile_id=` — кількість непрочитаних (для badge)
4. Додай in api.ts: getNotifications(), markNotificationRead(), getUnreadCount()
5. Створи `apps/web/src/pages/Notifications.tsx` — список notifications
6. Підключи route /notifications в App.tsx
7. В AppShellNew.tsx: підключи bell icon до /notifications, показуй unread badge (з useQuery на unread-count)

Перевірка: /notifications відкривається; bell icon показує кількість непрочитаних; click → mark read.
```

---

### 3.2 — CV Tailoring

```
Job Copilot monorepo. Rust engine-api + Python ml + React web.

Задача: фіча "Tailor CV" — показує що додати/прибрати в резюме для конкретної вакансії.

Контекст:
- ML сервіс вже має fit analysis: /api/v1/fit/analyze повертає matched_terms, missing_terms, score, reasons
- JobDetails сторінка: `apps/web/src/pages/JobDetails.tsx` — там є Match tab
- Idea: використати missing_terms як основу для tailoring suggestions
- LLM: якщо TemplateEnrichmentProvider — генерувати structured suggestions без LLM
- Spec: `docs/04-development/ml-strategy.md`

Що зробити:
1. Прочитай `apps/ml/app/main.py` — знайди fit analyze endpoint, зрозумій структуру відповіді
2. Прочитай `apps/ml/app/llm_provider.py` — знайди enrichment методи
3. Додай endpoint `POST /api/v1/cv-tailoring` в ML сервіс:
   - Input: {profile_id, job_id}
   - Дістає fit analysis (matched/missing terms, reasons)
   - TemplateEnrichmentProvider: генерує structured suggestions:
     ```json
     {
       "add_to_cv": ["Mention React experience", "Add TypeScript to skills"],
       "emphasize": ["Your Node.js background matches", "Highlight API design"],
       "optional": ["Consider adding Docker experience"]
     }
     ```
   - OllamaProvider (якщо є): передає в LLM для кращих suggestions
4. Додай в engine-api або прямо в ML proxy endpoint
5. В `apps/web/src/pages/JobDetails.tsx`:
   - Додай кнопку "Tailor CV" (поруч з іншими feedback кнопками)
   - Click → modal/panel з suggestions
   - Кожен suggestion → copy button
   - Loading та error states

Перевірка: кнопка "Tailor CV" в JobDetails → modal з suggestions; suggestions базовані на gap аналізі.
```

---

### 3.3 — Global search (FTS)

```
Job Copilot monorepo. Rust engine-api + PostgreSQL + React web.

Задача: глобальний пошук через PostgreSQL Full-Text Search.

Контекст:
- Header вже має search placeholder — знайди в AppShellNew.tsx або схожому
- PostgreSQL 16: підтримує to_tsvector, to_tsquery, GIN indexes
- Міграція 2.1 вже додала GIN index на jobs(title, description_text)
- Шукаємо по: jobs (title + company), applications (job title), contacts (name)

Що зробити:
1. Прочитай `apps/engine-api/src/api/` — зрозумій pattern
2. Додай endpoint `GET /api/v1/search?q=<query>&limit=10`:
   - Пошук в jobs: WHERE to_tsvector('simple', title || ' ' || description_text) @@ plainto_tsquery('simple', $1) AND is_active = true
   - Пошук в profiles/applications: по job title
   - Повертає: { jobs: [...], applications: [...] } максимум limit результатів кожного
3. Додай в api.ts: globalSearch(query: string)
4. Додай в queryKeys.ts: search.results(query)
5. Створи `apps/web/src/components/GlobalSearch.tsx`:
   - Відкривається по Cmd+K (useEffect + keydown listener)
   - Input з debounce 300ms
   - Показує jobs і applications результати у dropdown
   - Click на результат → навігація до /jobs/:id або /applications/:id
   - Escape → закрити
6. Підключи в AppShellNew.tsx або де є search placeholder

Перевірка: Cmd+K відкриває overlay; пошук "react" повертає релевантні вакансії; результати клікабельні.
```

---

### 3.4 — Profile extensions (experience, salary, languages)

```
Job Copilot monorepo. Rust engine-api + React web.

Задача: додати нові поля до профілю — years_of_experience, salary_min/max/currency, languages.

Контекст:
- Profile DTO: `apps/engine-api/src/api/dto/profile.rs`
- Profile page: `apps/web/src/pages/Profile.tsx`
- Поля впливатимуть на scoring (salary fit вже реалізований в 1.6)
- Migration треба додати нові колонки до таблиці profiles

Що зробити:
1. Прочитай `apps/engine-api/src/api/dto/profile.rs` і `apps/engine-api/migrations/`
2. Створи міграцію:
   ```sql
   ALTER TABLE profiles ADD COLUMN years_of_experience INTEGER;
   ALTER TABLE profiles ADD COLUMN salary_min INTEGER;
   ALTER TABLE profiles ADD COLUMN salary_max INTEGER;
   ALTER TABLE profiles ADD COLUMN salary_currency TEXT DEFAULT 'USD';
   ALTER TABLE profiles ADD COLUMN languages JSONB DEFAULT '[]';
   ```
3. Оновити ProfileResponse, UpdateProfileRequest DTOs
4. Оновити repository save/find логіку
5. В Profile.tsx додай поля:
   - Years of experience: number input (optional)
   - Expected salary: min-max range + currency select (USD/EUR/UAH)
   - Languages: multi-select або chip input (Ukrainian, English, German, Polish)
6. Підключи до saveMutation в useProfilePage.ts

Перевірка: нові поля зберігаються і завантажуються; salary_min/max доступні для scoring.
```

---

## ФАЗА 4 — Ollama

---

### 4.1 — Ollama provider + retrain pipeline

```
Job Copilot monorepo. Python ML сервіс.

Задача: завершити OllamaEnrichmentProvider і автоматичний retrain reranker.

Контекст:
- Файл: `apps/ml/app/llm_provider.py` — в попередньому task (1.4) додали skeleton OllamaEnrichmentProvider
- Файл: `apps/ml/app/bootstrap_training.py` — в task (1.5) написали bootstrap script
- Тепер: реалізувати всі методи Ollama provider (profile_insights, job_fit_explanation, cover_letter_draft, etc.)
- Ollama API: POST /api/generate з {model, prompt, stream: false} → {response: string}

Що зробити:
1. Прочитай `apps/ml/app/llm_provider.py` — знайди всі методи TemplateEnrichmentProvider і OpenAIEnrichmentProvider
2. Реалізуй кожен метод в OllamaEnrichmentProvider:
   - Підготуй промпт (стислий, structured output)
   - POST до Ollama
   - Парс відповіді (Ollama повертає plain text, треба витягти JSON якщо потрібно)
   - Fallback до TemplateEnrichmentProvider якщо Ollama недоступна
3. Для JSON output від Ollama — додай prompt prefix "Respond with valid JSON only:" і парс з try/except → fallback
4. Додай endpoint `POST /api/v1/reranker/retrain` в main.py:
   - Викликає bootstrap_training.py логіку
   - Повертає {status, examples_count, model_updated}
5. Додай `GET /api/v1/reranker/status` — повертає {examples_count, last_trained_at, model_loss}
6. Додай в docker-compose.yml секцію для ollama service (optional, commented by default)

Перевірка: ML_LLM_PROVIDER=ollama + запущений Ollama → enrichment endpoints повертають LLM-генерований текст.
```

---

## ФАЗА 5 — Платна підписка

---

### 5.1 — Auth: registration + login + JWT

```
Job Copilot monorepo. Rust engine-api.

Задача: додати базову auth — registration, login, JWT tokens.

Контекст:
- Зараз немає auth, profile_id береться з localStorage
- Треба: POST /api/v1/auth/register, POST /api/v1/auth/login → JWT
- JWT повинен містити profile_id (або user_id що лінкується до profile)
- Захистити всі /api/v1/ endpoints (крім /auth/* і /market/* публічних)
- Файл: `apps/engine-api/src/` — знайди де middleware/routes

Що зробити:
1. Прочитай структуру `apps/engine-api/src/api/` і `Cargo.toml` (які crates вже є)
2. Додай crate: `jsonwebtoken` для JWT, `bcrypt` або `argon2` для паролів
3. Створи міграцію: таблиця users (id, email, password_hash, created_at, tier TEXT DEFAULT 'free')
4. Зв'яжи: users.id → profiles (додай user_id колонку або зроби 1:1)
5. Реалізуй:
   - POST /api/v1/auth/register → {email, password} → create user + profile → JWT
   - POST /api/v1/auth/login → {email, password} → verify → JWT
   - JWT middleware: читає Authorization: Bearer <token>, validates, додає user_id в request context
6. Захисти всі існуючі endpoints (крім публічних)
7. Web: замінити localStorage profile_id на auth token в api.ts, додати login/register сторінки

Перевірка: register → login → JWT → authenticated requests працюють; неавторизований запит → 401.
```

---

### 5.2 — Tier system + paid LLM switching

```
Job Copilot monorepo. Rust engine-api + Python ml.

Задача: tier-based access — free (Ollama/template) vs paid (Claude/GPT).

Контекст:
- Попередній task (5.1) додав users.tier = 'free' | 'paid'
- ML provider вже перемикається через ML_LLM_PROVIDER env var
- Треба: paid users → LLM API виклики; free users → Ollama/template

Що зробити:
1. В engine-api: додай user tier до JWT payload
2. Коли ML enrichment endpoint викликається — передавай tier в запиті до ML сервісу
3. В ML сервісі: `get_enrichment_provider(tier: str)`:
   - tier='paid' + ANTHROPIC_API_KEY → AnthropicEnrichmentProvider (claude-haiku-4-5)
   - tier='paid' + OPENAI_API_KEY → OpenAIEnrichmentProvider
   - tier='free' + OLLAMA_BASE_URL → OllamaEnrichmentProvider
   - default → TemplateEnrichmentProvider
4. Додай AnthropicEnrichmentProvider в llm_provider.py:
   - Uses `anthropic` Python SDK (pip install anthropic)
   - Model: claude-haiku-4-5 (найдешевший, достатньо для цих tasks)
   - Same interface як OpenAI provider
5. Rate limiting: free tier → 10 LLM calls/day; paid → unlimited
6. В Web: показуй "Upgrade to Pro" для LLM фіч якщо tier='free' і Ollama недоступна

Перевірка: paid user отримує LLM response; free user — template/ollama; rate limit спрацьовує.
```
