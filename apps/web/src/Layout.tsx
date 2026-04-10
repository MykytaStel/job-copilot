import { useEffect, useRef, useState } from 'react';
import { NavLink, Outlet, useNavigate } from 'react-router-dom';
import { Toaster } from 'react-hot-toast';
import type { LucideIcon } from 'lucide-react';
import {
  Archive,
  Bell,
  Briefcase,
  FileText,
  KanbanSquare,
  LayoutDashboard,
  Loader,
  Mail,
  MessageSquare,
  PlusCircle,
  Scale,
  TrendingUp,
  Upload,
  User,
} from 'lucide-react';
import type { SearchResults } from '@job-copilot/shared';
import { search } from './api';

type NavLinkItem = { to: string; label: string; icon: LucideIcon };
type NavItem = NavLinkItem | null;

const links: NavItem[] = [
  { to: '/', label: 'Dashboard', icon: LayoutDashboard },
  { to: '/jobs/new', label: 'Add Job', icon: PlusCircle },
  { to: '/import', label: 'Batch Import', icon: Upload },
  { to: '/resume', label: 'Resume', icon: FileText },
  { to: '/applications', label: 'Applications', icon: KanbanSquare },
  { to: '/market', label: 'Market Pulse', icon: TrendingUp },
  { to: '/alerts', label: 'Alerts', icon: Bell },
  { to: '/compare', label: 'Compare', icon: Scale },
  { to: '/profile', label: 'Profile', icon: User },
  null, // divider
  { to: '/cover-letters', label: 'Cover Letters', icon: Mail },
  { to: '/interview-qa', label: 'Interview Q&A', icon: MessageSquare },
  { to: '/offers', label: 'Offers', icon: Briefcase },
  null, // divider
  { to: '/backup', label: 'Backup / Restore', icon: Archive },
];

export default function Layout() {
  return (
    <div className="appShell">
      <nav className="sidebar">
        <p className="navBrand">Job Copilot UA</p>
        <SearchBox />
        <ul className="navList">
          {links.map((link, i) =>
            link === null ? (
              <li key={`div-${i}`} style={{ height: 1, background: 'var(--border)', margin: '6px 0' }} />
            ) : (
              <li key={link.to}>
                <NavLink
                  to={link.to}
                  end={link.to === '/'}
                  className={({ isActive }) => (isActive ? 'navLink active' : 'navLink')}
                  style={{ display: 'flex', alignItems: 'center', gap: 8 }}
                >
                  <link.icon size={16} style={{ flexShrink: 0 }} />
                  {link.label}
                </NavLink>
              </li>
            )
          )}
        </ul>
      </nav>
      <main className="content">
        <Outlet />
      </main>
      <Toaster
        position="bottom-right"
        toastOptions={{
          duration: 3000,
          style: { fontSize: 14 },
        }}
      />
    </div>
  );
}

function SearchBox() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResults | null>(null);
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();

  // Close dropdown when clicking outside
  useEffect(() => {
    function onClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener('mousedown', onClickOutside);
    return () => document.removeEventListener('mousedown', onClickOutside);
  }, []);

  function handleChange(e: React.ChangeEvent<HTMLInputElement>) {
    const q = e.target.value;
    setQuery(q);

    if (debounceRef.current) clearTimeout(debounceRef.current);

    if (q.trim().length < 2) {
      setResults(null);
      setOpen(false);
      return;
    }

    debounceRef.current = setTimeout(async () => {
      setLoading(true);
      try {
        const res = await search(q.trim());
        setResults(res);
        setOpen(true);
      } catch {
        // silently ignore search errors
      } finally {
        setLoading(false);
      }
    }, 300);
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Escape') {
      setOpen(false);
      setQuery('');
    }
  }

  function goTo(url: string) {
    setOpen(false);
    setQuery('');
    setResults(null);
    navigate(url);
  }

  const hasResults =
    results && (results.jobs.length > 0 || results.contacts.length > 0);

  return (
    <div ref={containerRef} style={{ position: 'relative', margin: '8px 0 12px' }}>
      <input
        type="search"
        value={query}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        onFocus={() => hasResults && setOpen(true)}
        placeholder="Search…"
        style={{
          width: '100%',
          boxSizing: 'border-box',
          padding: '6px 10px',
          fontSize: 13,
          borderRadius: 6,
          border: '1px solid var(--border)',
          background: 'var(--surface, #f5f5f5)',
        }}
      />
      {loading && (
        <span style={{ position: 'absolute', right: 8, top: 7, color: 'var(--muted)', lineHeight: 0 }}>
          <Loader size={12} style={{ animation: 'spin 1s linear infinite' }} />
        </span>
      )}

      {open && hasResults && (
        <div
          style={{
            position: 'absolute',
            top: '100%',
            left: 0,
            right: 0,
            zIndex: 100,
            background: 'var(--bg, #fff)',
            border: '1px solid var(--border)',
            borderRadius: 6,
            boxShadow: '0 4px 16px rgba(0,0,0,0.12)',
            maxHeight: 320,
            overflowY: 'auto',
            marginTop: 4,
          }}
        >
          {results!.jobs.length > 0 && (
            <>
              <p style={{ margin: 0, padding: '6px 10px', fontSize: 10, fontWeight: 700, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: 1 }}>
                Jobs
              </p>
              {results!.jobs.map((job) => (
                <button
                  key={job.id}
                  type="button"
                  onClick={() => goTo(`/jobs/${job.id}`)}
                  style={{
                    display: 'block',
                    width: '100%',
                    textAlign: 'left',
                    padding: '7px 10px',
                    background: 'transparent',
                    border: 'none',
                    cursor: 'pointer',
                    fontSize: 13,
                    borderBottom: '1px solid var(--border)',
                  }}
                  onMouseEnter={(e) => (e.currentTarget.style.background = 'var(--surface, #f5f5f5)')}
                  onMouseLeave={(e) => (e.currentTarget.style.background = 'transparent')}
                >
                  <span style={{ fontWeight: 600 }}>{job.title}</span>
                  <span className="muted" style={{ marginLeft: 6, fontSize: 12 }}>{job.company}</span>
                </button>
              ))}
            </>
          )}

          {results!.contacts.length > 0 && (
            <>
              <p style={{ margin: 0, padding: '6px 10px', fontSize: 10, fontWeight: 700, color: 'var(--muted)', textTransform: 'uppercase', letterSpacing: 1 }}>
                Contacts
              </p>
              {results!.contacts.map((c) => (
                <button
                  key={c.id}
                  type="button"
                  onClick={() => goTo('/applications')}
                  style={{
                    display: 'block',
                    width: '100%',
                    textAlign: 'left',
                    padding: '7px 10px',
                    background: 'transparent',
                    border: 'none',
                    cursor: 'pointer',
                    fontSize: 13,
                    borderBottom: '1px solid var(--border)',
                  }}
                  onMouseEnter={(e) => (e.currentTarget.style.background = 'var(--surface, #f5f5f5)')}
                  onMouseLeave={(e) => (e.currentTarget.style.background = 'transparent')}
                >
                  <span style={{ fontWeight: 600 }}>{c.name}</span>
                  {c.role && <span className="muted" style={{ marginLeft: 6, fontSize: 12 }}>{c.role}</span>}
                  {c.email && <span className="muted" style={{ marginLeft: 6, fontSize: 12 }}>{c.email}</span>}
                </button>
              ))}
            </>
          )}
        </div>
      )}

      {open && results && !hasResults && query.length >= 2 && (
        <div
          style={{
            position: 'absolute',
            top: '100%',
            left: 0,
            right: 0,
            zIndex: 100,
            background: 'var(--bg, #fff)',
            border: '1px solid var(--border)',
            borderRadius: 6,
            padding: '10px',
            fontSize: 13,
            color: 'var(--muted)',
            marginTop: 4,
          }}
        >
          No results for "{query}"
        </div>
      )}
    </div>
  );
}
