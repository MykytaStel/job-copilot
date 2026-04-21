import { PillList } from '../../components/ui/PillList';

export function LatestAnalysisSection({ summary, skills }: { summary?: string; skills: string[] }) {
  return (
    <section className="space-y-5 rounded-[24px] border border-border bg-card/85 p-7">
      <div className="space-y-2">
        <p className="eyebrow">Latest analysis</p>
        <h3 className="m-0 text-lg font-semibold text-card-foreground">
          Persisted summary and extracted skills
        </h3>
      </div>
      {summary ? (
        <>
          <p className="m-0 leading-7 text-card-foreground">{summary}</p>
          <div className="resultSection">
            <PillList items={skills} emptyLabel="No skills returned." />
          </div>
        </>
      ) : (
        <p className="m-0 text-sm leading-6 text-muted-foreground">
          No persisted analysis yet. Save the profile, then run Analyze.
        </p>
      )}
    </section>
  );
}
