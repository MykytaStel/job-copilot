import type { ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { ChevronRight, type LucideIcon } from 'lucide-react';
import { Button } from './Button';
import { AccentIconFrame } from './AccentIconFrame';
import { cn } from '../../lib/cn';

interface SectionHeaderProps {
  title: string;
  description?: string;
  icon?: LucideIcon;
  action?: {
    label: string;
    href?: string;
    onClick?: () => void;
  };
  className?: string;
  children?: ReactNode;
}

export function SectionHeader({
  title,
  description,
  icon: Icon,
  action,
  className,
  children,
}: SectionHeaderProps) {
  if (!Icon && !description && !action) {
    // Backward-compatible: simple eyebrow label
    return <p className="eyebrow sectionHeader">{title}</p>;
  }

  return (
    <div
      className={cn(
        'flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between',
        className,
      )}
    >
      <div className="flex items-center gap-3">
        {Icon && (
          <AccentIconFrame size="md">
            <Icon className="h-5 w-5 text-primary" />
          </AccentIconFrame>
        )}
        <div>
          <h2 className="m-0 text-lg font-semibold text-foreground">{title}</h2>
          {description && (
            <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
          )}
        </div>
      </div>
      <div className="flex items-center gap-2">
        {children}
        {action?.href ? (
          <Link to={action.href} className="no-underline">
            <Button variant="ghost" size="sm" className="text-primary hover:text-primary">
              {action.label}
              <ChevronRight className="ml-1 h-4 w-4" />
            </Button>
          </Link>
        ) : action ? (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={action.onClick}
            className="text-primary hover:text-primary"
          >
            {action.label}
            <ChevronRight className="ml-1 h-4 w-4" />
          </Button>
        ) : null}
      </div>
    </div>
  );
}

interface PageHeaderProps {
  title: string;
  description?: string;
  breadcrumb?: { label: string; href?: string }[];
  actions?: ReactNode;
  className?: string;
}

export function PageHeader({
  title,
  description,
  breadcrumb,
  actions,
  className,
}: PageHeaderProps) {
  return (
    <div className={cn('space-y-3', className)}>
      {breadcrumb && breadcrumb.length > 0 && (
        <nav className="flex items-center gap-1 text-sm text-muted-foreground">
          {breadcrumb.map((item, index) => (
            <span key={`${item.label}-${index}`} className="flex items-center gap-1">
              {index > 0 && <ChevronRight className="h-3 w-3" />}
              {item.href ? (
                <Link
                  to={item.href}
                  className="transition-colors hover:text-foreground no-underline"
                >
                  {item.label}
                </Link>
              ) : (
                <span>{item.label}</span>
              )}
            </span>
          ))}
        </nav>
      )}
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="m-0 text-2xl font-bold tracking-tight text-foreground">{title}</h1>
          {description && (
            <p className="mt-2 mb-0 max-w-3xl text-sm leading-6 text-muted-foreground">
              {description}
            </p>
          )}
        </div>
        {actions && <div className="flex items-center gap-2">{actions}</div>}
      </div>
    </div>
  );
}
