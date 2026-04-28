import type { RoleCatalogItem } from '../api/profiles';

type ProfileStrengthProps = {
  skills: string[];
  roles: RoleCatalogItem[];
  activeJobsCount: number;
};

const ROLE_SKILL_HINTS: Record<string, string[]> = {
  frontend_engineer: ['react', 'typescript', 'javascript', 'css', 'html', 'vue', 'angular'],
  backend_engineer: ['node', 'rust', 'go', 'python', 'java', 'api', 'postgres', 'sql'],
  fullstack_engineer: ['react', 'typescript', 'node', 'api', 'postgres', 'sql'],
  mobile_engineer: ['react native', 'ios', 'android', 'swift', 'kotlin', 'expo'],
  devops_engineer: ['aws', 'docker', 'kubernetes', 'terraform', 'ci/cd', 'linux'],
  data_engineer: ['sql', 'python', 'etl', 'spark', 'airflow', 'analytics'],
  ml_engineer: ['python', 'machine learning', 'pytorch', 'tensorflow', 'nlp', 'ai'],
  qa_engineer: ['qa', 'testing', 'automation', 'playwright', 'cypress', 'selenium'],
  product_designer: ['figma', 'ux', 'ui', 'research', 'prototyping'],
  product_manager: ['product', 'roadmap', 'analytics', 'discovery', 'strategy'],
  project_manager: ['project management', 'scrum', 'agile', 'delivery', 'jira'],
  tech_lead: ['architecture', 'leadership', 'mentoring', 'system design'],
  engineering_manager: ['leadership', 'management', 'hiring', 'delivery'],
};

export function ProfileStrength({ skills, roles, activeJobsCount }: ProfileStrengthProps) {
  const matchCount = roles.filter((role) => isStrongRoleMatch(role, skills)).length;

  return (
    <div className="space-y-2">
      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
        Profile Strength
      </p>
      <p className="m-0 text-sm font-semibold text-card-foreground">
        Your profile is a strong match for {matchCount} roles in the database.
      </p>
      <p className="m-0 text-xs text-muted-foreground">You match {activeJobsCount} active jobs</p>
    </div>
  );
}

function isStrongRoleMatch(role: RoleCatalogItem, skills: string[]): boolean {
  if (role.isFallback || skills.length === 0) return false;

  const roleHints = ROLE_SKILL_HINTS[role.id] ?? role.displayName.split(/\s+/);
  const normalizedSkills = skills.map(normalize);
  const matchedHints = roleHints.filter((hint) => {
    const normalizedHint = normalize(hint);
    return normalizedSkills.some(
      (skill) => skill.includes(normalizedHint) || normalizedHint.includes(skill),
    );
  });

  return matchedHints.length >= Math.min(2, roleHints.length);
}

function normalize(value: string): string {
  return value.trim().toLowerCase();
}
