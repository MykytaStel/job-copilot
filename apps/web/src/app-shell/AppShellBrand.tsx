import { Link } from 'react-router-dom';
import { Sparkles } from 'lucide-react';

export function AppShellBrand({ onClick }: { onClick?: () => void }) {
  return (
    <Link to="/" className="flex items-center gap-2 no-underline" onClick={onClick}>
      <div
        className="flex h-8 w-8 items-center justify-center rounded-lg"
        style={{ background: 'var(--gradient-button)' }}
      >
        <Sparkles className="h-4 w-4 text-white" />
      </div>
      <span className="text-lg font-semibold text-sidebar-foreground">Job Copilot</span>
    </Link>
  );
}

export function AppShellBrandMark() {
  return (
    <div
      className="flex h-8 w-8 items-center justify-center rounded-lg"
      style={{ background: 'var(--gradient-button)' }}
    >
      <Sparkles className="h-4 w-4 text-white" />
    </div>
  );
}
