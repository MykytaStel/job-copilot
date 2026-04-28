import { BriefcaseBusiness, FileText, Plus, Target } from 'lucide-react';
import {
  LatestAnalysisSection,
  ProfileFormSection,
  RankedResultsSection,
  SearchProfileBuilderSection,
  SearchProfileResultSection,
} from '../features/profile/ProfileSections';
import { Badge } from '../components/ui/Badge';
import { AccentIconFrame } from '../components/ui/AccentIconFrame';
import { Card, CardContent } from '../components/ui/Card';
import { Page, PageGrid } from '../components/ui/Page';
import { ProfileCompletion } from '../components/ProfileCompletion';
import { Button } from '../components/ui/Button';
import { PageHeader } from '../components/ui/SectionHeader';
import { SurfaceMetric } from '../components/ui/Surface';
import { useProfilePage } from '../features/profile/useProfilePage';
import { ProfileStrength } from '../components/ProfileStrength';
import { ResumeHistory } from '../components/ResumeHistory';

export default function Profile() {
  const {
    fileInputRef,
    profile,
    roles,
    resumes,
    activeJobsCount,
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
    preferredLocations,
    experience,
    workModePreference,
    profileCompletion,
    targetRegions,
    workModes,
    preferredRoles,
    allowedSources,
    includeKeywordsInput,
    excludeKeywordsInput,
    buildResult,
    buildIsCurrent,
    buildRestoredFromStorage,
    searchResult,
    searchError,
    suggestedSkills,
    saveMutation,
    analyzeMutation,
    updateSkillsMutation,
    activateResumeMutation,
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
    setWorkModePreference,
    addPreferredLocation,
    removePreferredLocation,
    addExperience,
    updateExperience,
    removeExperience,
    setIncludeKeywordsInput,
    setExcludeKeywordsInput,
    addLanguage,
    removeLanguage,
    updateLanguageLevel,
    toggleTargetRegion,
    toggleWorkMode,
    togglePreferredRole,
    toggleAllowedSource,
    saveCurrentProfile,
    buildCurrentSearchProfile,
    runCurrentSearch,
    analyzeProfile,
    addSuggestedSkill,
    addAllSuggestedSkills,
    openFilePicker,
    handleFileChange,
    portfolioUrl,
    githubUrl,
    linkedinUrl,
    setPortfolioUrl,
    setGithubUrl,
    setLinkedinUrl,
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
                <SurfaceMetric>
                  <div className="flex items-center gap-3">
                    <AccentIconFrame size="md">
                      <FileText className="h-4 w-4" />
                    </AccentIconFrame>
                    <div>
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Profile
                      </p>
                      <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">
                        {profile ? 'Persisted' : 'Draft only'}
                      </p>
                    </div>
                  </div>
                </SurfaceMetric>
                <SurfaceMetric>
                  <div className="flex items-center gap-3">
                    <AccentIconFrame size="md">
                      <BriefcaseBusiness className="h-4 w-4" />
                    </AccentIconFrame>
                    <div>
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Role catalog
                      </p>
                      <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">
                        {roles.length} roles loaded
                      </p>
                    </div>
                  </div>
                </SurfaceMetric>
                <SurfaceMetric>
                  <div className="flex items-center gap-3">
                    <AccentIconFrame size="md">
                      <Target className="h-4 w-4" />
                    </AccentIconFrame>
                    <div>
                      <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
                        Search results
                      </p>
                      <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">
                        {searchResult ? `${searchResult.meta.returnedJobs} ranked` : 'Not run yet'}
                      </p>
                    </div>
                  </div>
                </SurfaceMetric>
                <SurfaceMetric className="sm:col-span-3">
                  <ProfileCompletion completion={profileCompletion} />
                </SurfaceMetric>
                <SurfaceMetric className="sm:col-span-3">
                  <ProfileStrength
                    skills={profile?.skills ?? []}
                    roles={roles}
                    activeJobsCount={activeJobsCount}
                  />
                </SurfaceMetric>
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
            preferredLocations={preferredLocations}
            experience={experience}
            workModePreference={workModePreference}
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
            setWorkModePreference={setWorkModePreference}
            onAddPreferredLocation={addPreferredLocation}
            onRemovePreferredLocation={removePreferredLocation}
            onAddExperience={addExperience}
            onUpdateExperience={updateExperience}
            onRemoveExperience={removeExperience}
            onAddLanguage={addLanguage}
            onRemoveLanguage={removeLanguage}
            onUpdateLanguageLevel={updateLanguageLevel}
            portfolioUrl={portfolioUrl}
            githubUrl={githubUrl}
            linkedinUrl={linkedinUrl}
            setPortfolioUrl={setPortfolioUrl}
            setGithubUrl={setGithubUrl}
            setLinkedinUrl={setLinkedinUrl}
          />

          <ResumeHistory
            resumes={resumes}
            activatingResumeId={
              activateResumeMutation.isPending ? activateResumeMutation.variables : undefined
            }
            onActivate={(resumeId) => activateResumeMutation.mutate(resumeId)}
          />

          {suggestedSkills.length > 0 && (
            <Card className="border-border bg-card">
              <CardContent className="space-y-4 p-5">
                <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                  <div>
                    <p className="eyebrow">Suggested skills</p>
                    <h3 className="m-0 text-base font-semibold text-card-foreground">
                      Extracted from the latest analysis
                    </h3>
                  </div>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={addAllSuggestedSkills}
                    disabled={updateSkillsMutation.isPending}
                  >
                    Add all
                  </Button>
                </div>
                <div className="flex flex-wrap gap-2">
                  {suggestedSkills.map((skill) => (
                    <span
                      key={skill}
                      className="inline-flex items-center gap-2 rounded-full border border-border bg-surface-muted px-3 py-1.5 text-xs font-medium text-card-foreground"
                    >
                      {skill}
                      <Button
                        type="button"
                        variant="icon"
                        size="icon"
                        className="h-6 w-6 rounded-full"
                        onClick={() => addSuggestedSkill(skill)}
                        disabled={updateSkillsMutation.isPending}
                        aria-label={`Add ${skill}`}
                        title={`Add ${skill}`}
                      >
                        <Plus className="h-3.5 w-3.5" />
                      </Button>
                    </span>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          <SearchProfileBuilderSection
            profileExists={Boolean(profile)}
            hasPersistedPreferences={Boolean(profile?.searchPreferences)}
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
        <SearchProfileResultSection
          result={buildResult}
          roles={roles}
          sources={sources}
          statusMessage={
            !buildIsCurrent
              ? 'Current raw text or filters no longer match this build. Rebuild before running ranking.'
              : buildRestoredFromStorage
                ? 'Restored the last built search profile for these inputs.'
                : undefined
          }
        />
      )}

      {buildResult && (
        <RankedResultsSection
          searchResult={searchResult}
          searchError={searchError}
          roles={roles}
          sources={sources}
          buildResult={buildResult}
          buildIsCurrent={buildIsCurrent}
          buildRestoredFromStorage={buildRestoredFromStorage}
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
