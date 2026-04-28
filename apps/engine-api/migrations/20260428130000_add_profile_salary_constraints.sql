UPDATE profiles
SET salary_currency = UPPER(TRIM(salary_currency))
WHERE salary_currency IS NOT NULL;

UPDATE profiles
SET salary_currency = NULL
WHERE salary_currency IS NOT NULL
  AND salary_currency NOT IN ('USD', 'EUR', 'UAH');

UPDATE profiles
SET salary_min = salary_max,
    salary_max = salary_min
WHERE salary_min IS NOT NULL
  AND salary_max IS NOT NULL
  AND salary_min > salary_max;

ALTER TABLE profiles
    ADD CONSTRAINT profiles_salary_bounds_check
    CHECK (
        salary_min IS NULL
        OR salary_max IS NULL
        OR salary_min <= salary_max
    );

ALTER TABLE profiles
    ADD CONSTRAINT profiles_salary_currency_check
    CHECK (
        salary_currency IS NULL
        OR salary_currency IN ('USD', 'EUR', 'UAH')
    );
