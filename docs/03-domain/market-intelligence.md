# Market Intelligence — Специфікація

> Ідея: система вже збирає тисячі вакансій. Замість просто показувати їх юзеру —
> агрегуємо дані і показуємо картину ринку. Без додаткових витрат.

## Проблема яку вирішуємо

Кандидат не знає:
- Яка компанія активно набирає (vs. заморозила hiring)
- Які зарплати реальні для його рівня в 2026
- Які технології зараз in demand
- Скільки відкритих вакансій у його ніші
- Де концентруються хороші remote позиції

Ці дані є в скрапнутих вакансіях — їх просто ніхто не агрегує.

## Що збираємо (всі дані вже є в БД)

### 1. Company Hiring Velocity
**Питання:** Хто активно набирає? Хто зупинився?

```sql
SELECT company_name,
       COUNT(*) FILTER (WHERE is_active AND last_seen_at > NOW() - INTERVAL '7 days') AS jobs_this_week,
       COUNT(*) FILTER (WHERE is_active AND last_seen_at > NOW() - INTERVAL '30 days') AS jobs_this_month,
       COUNT(*) FILTER (WHERE inactivated_at > NOW() - INTERVAL '14 days') AS jobs_closed_recently
FROM jobs
GROUP BY company_name
HAVING COUNT(*) > 2
ORDER BY jobs_this_week DESC;
```

**Сигнали:**
- `jobs_this_week > 5` → компанія активно росте
- `jobs_closed_recently > jobs_this_week` → можливе скорочення / hiring freeze
- `last_seen_at` для компанії давно → призупинили набір

### 2. Salary Trends (по role + seniority)
**Питання:** Яка реальна зарплата для Senior React Dev у 2026?

```sql
SELECT seniority, source,
       PERCENTILE_CONT(0.25) WITHIN GROUP (ORDER BY salary_min) AS p25,
       PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY salary_min) AS median,
       PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY salary_min) AS p75,
       COUNT(*) AS sample_count
FROM jobs
WHERE salary_min IS NOT NULL
  AND last_seen_at > NOW() - INTERVAL '30 days'
  AND is_active = TRUE
GROUP BY seniority, source;
```

**Показуємо:** медіану, p25/p75 діапазон, кількість вакансій з salary data.

### 3. Role Demand Index
**Питання:** Які ролі зростають, які падають?

```sql
-- Порівняємо цей тиждень з минулим
SELECT 
    CASE 
        WHEN title ILIKE '%frontend%' OR title ILIKE '%react%' THEN 'Frontend'
        WHEN title ILIKE '%backend%' OR title ILIKE '%python%' THEN 'Backend'
        WHEN title ILIKE '%devops%' OR title ILIKE '%infrastructure%' THEN 'DevOps'
        WHEN title ILIKE '%data%' OR title ILIKE '%analyst%' THEN 'Data'
        -- TODO: використати RoleId catalog коли буде готовий
    END AS role_group,
    COUNT(*) FILTER (WHERE first_seen_at > NOW() - INTERVAL '7 days') AS new_this_week,
    COUNT(*) FILTER (WHERE first_seen_at > NOW() - INTERVAL '14 days' 
                       AND first_seen_at <= NOW() - INTERVAL '7 days') AS new_prev_week
FROM jobs WHERE is_active = TRUE
GROUP BY role_group;
```

### 4. Market Freeze Signals
**Питання:** Які компанії могли провести звільнення?

Алгоритм:
1. Компанія мала > 5 активних вакансій 30 днів тому
2. Зараз має < 2 активних вакансій
3. Більшість їхніх вакансій позначені `inactivated_at` в короткому проміжку (< 3 дні)

Сигнал: "ймовірна заморозка або скорочення".
Показувати обережно (не стверджувати, а позначати як signal).

### 5. Tech Skills Demand
**Питання:** Які технології зараз найпопулярніші?

```sql
-- З `jobs.description_text` та engine-api skill extraction
-- Використовує вже нормалізовані терміни з matching engine
SELECT skill, COUNT(*) AS frequency,
       COUNT(*) FILTER (WHERE first_seen_at > NOW() - INTERVAL '7 days') AS new_this_week
FROM job_skills_view  -- materialized view або computed
GROUP BY skill
ORDER BY frequency DESC
LIMIT 50;
```

### 6. Remote Work Adoption
```sql
SELECT remote_type, source, COUNT(*) AS count
FROM jobs WHERE is_active = TRUE AND last_seen_at > NOW() - INTERVAL '30 days'
GROUP BY remote_type, source;
```

## Нова таблиця: `market_snapshots`

```sql
CREATE TABLE market_snapshots (
    id TEXT PRIMARY KEY,
    snapshot_date DATE NOT NULL,
    snapshot_type TEXT NOT NULL,  -- 'company_stats' | 'salary_trends' | 'role_demand' | 'skill_demand'
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX ON market_snapshots(snapshot_date, snapshot_type);
```

**Заповнення:** раз на день (або після кожного ingestion run) — окремий job в ingestion daemon.

## Нові API endpoints

```
GET /api/v1/market/companies?limit=20&sort=velocity   → top hiring companies
GET /api/v1/market/salaries?role=backend&seniority=senior → salary distribution
GET /api/v1/market/roles?period=30d                   → role demand trends
GET /api/v1/market/freeze-signals                     → companies with sudden drop
GET /api/v1/market/skills?limit=30                    → top skills demand
GET /api/v1/market/overview                           → summary for dashboard widget
```

## Web: Market Intelligence сторінка

**Route:** `/market`

**Секції:**
1. **Market Overview** — 4 stat cards: нових вакансій цього тижня, активних компаній, медіана зарплати, % remote
2. **Top Hiring Companies** — bar chart + table (company, jobs count, velocity trend)
3. **Salary Distribution** — по role/seniority: box plot або median+range
4. **Role Demand** — area chart: Frontend / Backend / DevOps / Data по тижнях
5. **Freeze Signals** — "These companies may have paused hiring" (з disclaimer)
6. **Hot Skills** — tag cloud або bar chart: top 20 skills цього місяця
7. **Personal vs Market** — порівняння skills юзера з market demand (якщо profile є)

## Чому це диференціатор

- **Djinni Analytics** показує загальні статистики, не per-candidate
- **Work.ua статистика** — сира, не персоналізована
- **Наш підхід**: market data + your profile = "ти в правильному місці" або "треба вчити X"
- Залучає юзерів навіть без профілю (публічна аналітика → конвертація в реєстрацію)

## Важливо

- Дані мають відображати реальний ринок, не домислення
- Для salary: показувати sample_count і disclaimer "Based on X job postings"
- Для freeze signals: м'яке формулювання ("may have paused", not "is firing")
- Оновлення: щоденно (не реального часу — надто дорого і не потрібно)
