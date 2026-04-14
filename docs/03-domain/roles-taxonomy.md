# Role Taxonomy

Use one canonical role catalog in the domain layer.

Current role set:
- react_native_developer
- frontend_developer
- backend_developer
- fullstack_developer
- qa_engineer
- devops_engineer
- data_analyst
- ui_ux_designer
- product_manager
- project_manager
- marketing_specialist
- sales_manager
- customer_support_specialist
- recruiter
- generalist

## Rules
- internal code should prefer `RoleId`
- API may still expose snake_case string keys
- scoring rules should point to canonical role IDs
- search aliases belong to the role catalog, not only heuristics
