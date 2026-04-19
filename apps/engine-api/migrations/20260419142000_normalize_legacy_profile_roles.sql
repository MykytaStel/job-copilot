UPDATE profiles
SET primary_role = CASE LOWER(TRIM(primary_role))
    WHEN 'react_native_developer' THEN 'mobile_engineer'
    WHEN 'frontend_developer' THEN 'frontend_engineer'
    WHEN 'backend_developer' THEN 'backend_engineer'
    WHEN 'fullstack_developer' THEN 'fullstack_engineer'
    WHEN 'ui_ux_designer' THEN 'product_designer'
    WHEN 'data_analyst' THEN 'data_engineer'
    WHEN 'marketing_specialist' THEN 'generalist'
    WHEN 'sales_manager' THEN 'generalist'
    WHEN 'customer_support_specialist' THEN 'generalist'
    WHEN 'recruiter' THEN 'generalist'
    ELSE LOWER(TRIM(primary_role))
END
WHERE primary_role IS NOT NULL;
