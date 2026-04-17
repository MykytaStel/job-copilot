import {
  LatestAnalysisSection,
  ProfileFormSection,
  RankedResultsSection,
  SearchProfileBuilderSection,
  SearchProfileResultSection,
} from '../features/profile/ProfileSections';
import { Page } from '../components/ui/Page';
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
    setIncludeKeywordsInput,
    setExcludeKeywordsInput,
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
      <ProfileFormSection
        name={name}
        email={email}
        location={location}
        rawText={rawText}
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

      {profile && (
        <LatestAnalysisSection summary={profile.summary} skills={profile.skills} />
      )}
    </Page>
  );
}
