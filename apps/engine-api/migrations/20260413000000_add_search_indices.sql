ALTER TABLE jobs
ADD COLUMN IF NOT EXISTS search_vector tsvector;

UPDATE jobs
SET search_vector = to_tsvector(
    'simple',
    concat_ws(
        ' ',
        coalesce(title, ''),
        coalesce(company_name, ''),
        coalesce(location, ''),
        coalesce(remote_type, ''),
        coalesce(seniority, ''),
        coalesce(description_text, '')
    )
)
WHERE search_vector IS NULL;

CREATE OR REPLACE FUNCTION jobs_search_vector_update() RETURNS trigger AS $$
BEGIN
    NEW.search_vector :=
        to_tsvector(
            'simple',
            concat_ws(
                ' ',
                coalesce(NEW.title, ''),
                coalesce(NEW.company_name, ''),
                coalesce(NEW.location, ''),
                coalesce(NEW.remote_type, ''),
                coalesce(NEW.seniority, ''),
                coalesce(NEW.description_text, '')
            )
        );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS jobs_search_vector_update_trigger ON jobs;

CREATE TRIGGER jobs_search_vector_update_trigger
BEFORE INSERT OR UPDATE OF title, company_name, location, remote_type, seniority, description_text
ON jobs
FOR EACH ROW
EXECUTE FUNCTION jobs_search_vector_update();

CREATE INDEX IF NOT EXISTS jobs_search_vector_idx ON jobs USING GIN (search_vector);
CREATE INDEX IF NOT EXISTS jobs_title_idx ON jobs (title);
CREATE INDEX IF NOT EXISTS jobs_company_name_idx ON jobs (company_name);
