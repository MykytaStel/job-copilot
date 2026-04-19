import { BriefcaseBusiness, FileText, Target } from 'lucide-react';
import {
  LatestAnalysisSection,
  ProfileFormSection,
  RankedResultsSection,
  SearchProfileBuilderSection,
  SearchProfileResultSection,
} from '../features/profile/ProfileSections';
import { Badge } from '../components/ui/Badge';
import { Card, CardContent } from '../components/ui/Card';
import { Page, PageGrid } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { useProfilePage } from '../features/profile/useProfilePage';

export default function Profile() {
  const {
    fileInputRef,
    profile,
    roles,
    sources,
    rolesError,
    sourcesError,
    llmContext,
    llmContextError,
    llmContextLoading,
    name,
    email,
    location,
    rawText,
    yearsOfExperience,
    salaryMin,
    salaryMax,
    salaryCurrency,
    languages,
    targetRegions,
    workModes,
    preferredRoles,
    allowedSources,
    includeKeywordsInput,
    excludeKeywordsInput,
    buildResult,
    searchResult,
    searchError,
    saveMutation,
    analyzeMutation,
    buildMutation,
    runMutation,
    setName,
    setEmail,
    setLocation,
    setRawText,
    setYearsOfExperience,
    setSalaryMin,
    setSalaryMax,
    setSalaryCurrency,
    setIncludeKeywordsInput,
    setExcludeKeywordsInput,
    toggleLanguage,
    toggleTargetRegion,
    toggleWorkMode,
    togglePreferredRole,
    toggleAllowedSource,
    saveCurrentProfile,
    buildCurrentSearchProfile,
    runCurrentSearch,
    analyzeProfile,
    openFilePicker,
    handleFileChange,
  } = useProfilePage();

  return (
    <Page>
      <PageHeader
        title="Profile & Search"
        description="Manage the persisted candidate profile, build a structured search profile, and inspect ranked results with explanation layers."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Profile' }]}
      />

      <Card className="overflow-hidden border-border bg-card">
        <CardContent className="p-0">
          <div className="relative">
            <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/10 via-accent/6 to-transparent" />
            <div className="relative flex flex-col gap-6 p-7 lg:flex-row lg:items-end lg:justify-between">
              <div className="max-w-3xl space-y-3">
                <div className="flex flex-wrap gap-2">
                  <Badge
                    variant="default"
                    className="border-0 bg-primary/15 px-2 py-0.5 text-xs text-primary"
                  >
                    Search intent
                  </Badge>
                  <Badge
                    variant="muted"
                    className="px-2 py-0.5 text-[10px] uppercase tracking-wide"
                  >
                    Persisted profile, structured filters, ranked results
                  </Badge>
                </div>
                <h2 className="m-0 text-2xl font-bold text-card-foreground lg:text-3xl">
                  Build a candidate profile once, then reuse it across ranking and AI assistance
                </h2>
                <p className="m-0 text-sm leading-7 text-muted-foreground lg:text-base">
                  Keep the canonical profile in sync, shape structured search preferences, and
                  inspect deterministic matches before asking for explanation or coaching.
                </p>
              </div>
              <div className="grid gap-3 sm:grid-cols-3 lg:min-w-[460px]">
                <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-primary/15 bg-primary/10 text-primary">
                      <FileText className="h-4 w-4" />
                    </div>
                    <div>
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Profile
                      </p>
                      <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">
                        {profile ? 'Persisted' : 'Draft only'}
                      </p>
                    </div>
                  </div>
                </div>
                <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-primary/15 bg-primary/10 text-primary">
                      <BriefcaseBusiness className="h-4 w-4" />
                    </div>
                    <div>
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Role catalog
                      </p>
                      <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">
                        {roles.length} roles loaded
                      </p>
                    </div>
                  </div>
                </div>
                <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-primary/15 bg-primary/10 text-primary">
                      <Target className="h-4 w-4" />
                    </div>
                    <div>
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Search results
                      </p>
                      <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">
                        {searchResult ? `${searchResult.meta.returnedJobs} ranked` : 'Not run yet'}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <PageGrid
        aside={
          profile ? (
            <LatestAnalysisSection summary={profile.summary} skills={profile.skills} />
          ) : undefined
        }
      >
        <div className="space-y-8">
          <ProfileFormSection
            name={name}
            email={email}
            location={location}
            rawText={rawText}
            yearsOfExperience={yearsOfExperience}
            salaryMin={salaryMin}
            salaryMax={salaryMax}
            salaryCurrency={salaryCurrency}
            languages={languages}
            profileExists={Boolean(profile)}
            fileInputRef={fileInputRef}
            isSaving={saveMutation.isPending}
            isAnalyzing={analyzeMutation.isPending}
            onSave={saveCurrentProfile}
            onAnalyze={analyzeProfile}
            onOpenFilePicker={openFilePicker}
            onFileChange={handleFileChange}
            setName={setName}
            setEmail={setEmail}
            setLocation={setLocation}
            setRawText={setRawText}
            setYearsOfExperience={setYearsOfExperience}
            setSalaryMin={setSalaryMin}
            setSalaryMax={setSalaryMax}
            setSalaryCurrency={setSalaryCurrency}
            onToggleLanguage={toggleLanguage}
          />

          <SearchProfileBuilderSection
            targetRegions={targetRegions}
            workModes={workModes}
            preferredRoles={preferredRoles}
            allowedSources={allowedSources}
            includeKeywordsInput={includeKeywordsInput}
            excludeKeywordsInput={excludeKeywordsInput}
            roles={roles}
            sources={sources}
            rolesError={rolesError}
            sourcesError={sourcesError}
            isBuilding={buildMutation.isPending}
            canBuild={Boolean(rawText.trim())}
            onBuild={buildCurrentSearchProfile}
            onToggleTargetRegion={toggleTargetRegion}
            onToggleWorkMode={toggleWorkMode}
            onTogglePreferredRole={togglePreferredRole}
            onToggleAllowedSource={toggleAllowedSource}
            setIncludeKeywordsInput={setIncludeKeywordsInput}
            setExcludeKeywordsInput={setExcludeKeywordsInput}
          />
        </div>
      </PageGrid>

      {buildResult && (
        <SearchProfileResultSection result={buildResult} roles={roles} sources={sources} />
      )}

      {buildResult && (
        <RankedResultsSection
          searchResult={searchResult}
          searchError={searchError}
          roles={roles}
          sources={sources}
          buildResult={buildResult}
          profileId={profile?.id ?? null}
          rawProfileText={rawText}
          llmContext={llmContext}
          llmContextError={llmContextError}
          llmContextLoading={llmContextLoading}
          isRunning={runMutation.isPending}
          onRunSearch={runCurrentSearch}
        />
      )}
    </Page>
  );
}
