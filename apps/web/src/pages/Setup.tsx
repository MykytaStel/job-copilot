import { CheckCircle2, Sparkles, ArrowRight } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';

import { getProfile } from '../api/profiles';
import { Button } from '../components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import { Page } from '../components/ui/Page';
import { markOnboardingSeen } from '../app-shell/useAppShell';
import { queryKeys } from '../queryKeys';

export default function Setup() {
  const navigate = useNavigate();

  const { data: profile } = useQuery({
    queryKey: queryKeys.profile.root(),
    queryFn: getProfile,
  });

  function handleContinue() {
    if (profile?.id) {
      markOnboardingSeen(profile.id);
    }
    navigate('/', { replace: true });
  }

  const steps = [
    { label: 'Profile created', done: true },
    { label: 'CV text stored', done: !!profile },
    { label: 'ML analysis running in background', done: false, pending: true },
    { label: 'Ranking and fit explanations ready', done: false, pending: true },
  ];

  return (
    <Page>
      <div className="mx-auto max-w-lg pt-8">
        <div className="mb-8 text-center">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/15">
            <Sparkles className="h-6 w-6 text-primary" />
          </div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            Welcome to Job Copilot
          </h1>
          <p className="mt-2 text-sm text-muted-foreground">
            Your profile has been created. Here is what happens next.
          </p>
        </div>

        <Card className="border-border bg-card">
          <CardHeader>
            <CardTitle>Getting started</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {steps.map((step) => (
              <div key={step.label} className="flex items-center gap-3">
                {step.done ? (
                  <CheckCircle2 className="h-4 w-4 shrink-0 text-emerald-400" />
                ) : step.pending ? (
                  <span className="h-4 w-4 shrink-0 rounded-full border-2 border-border/60 border-t-primary animate-spin" />
                ) : (
                  <span className="h-4 w-4 shrink-0 rounded-full border border-border/40" />
                )}
                <span
                  className={
                    step.done
                      ? 'text-sm text-foreground'
                      : 'text-sm text-muted-foreground'
                  }
                >
                  {step.label}
                </span>
              </div>
            ))}
          </CardContent>
        </Card>

        <p className="mt-4 text-center text-xs text-muted-foreground">
          Analysis runs in the background — you can use the dashboard while it completes.
        </p>

        <div className="mt-6 flex justify-center">
          <Button onClick={handleContinue} className="gap-2">
            Continue to Dashboard
            <ArrowRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </Page>
  );
}
