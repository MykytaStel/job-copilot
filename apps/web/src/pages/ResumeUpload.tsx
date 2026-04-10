import { useRef, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { activateResume, getResumes, uploadResume, uploadResumeFile } from '../api';
import { queryKeys } from '../queryKeys';

export default function ResumeUpload() {
  const queryClient = useQueryClient();
  const [text, setText] = useState('');
  const [filename, setFilename] = useState('resume.txt');
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const { data: versions = [] } = useQuery({
    queryKey: queryKeys.resumes.all(),
    queryFn: async () => {
      const list = await getResumes();
      return [...list].sort((a, b) => b.version - a.version);
    },
  });

  const uploadMutation = useMutation({
    mutationFn: () =>
      selectedFile ? uploadResumeFile(selectedFile) : uploadResume({ filename, rawText: text }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.resumes.all() });
      setText('');
      setSelectedFile(null);
      setError(null);
      if (fileRef.current) fileRef.current.value = '';
    },
    onError: (err: unknown) => setError(err instanceof Error ? err.message : 'Error'),
  });

  const activateMutation = useMutation({
    mutationFn: (id: string) => activateResume(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.resumes.all() });
      setError(null);
    },
    onError: (err: unknown) => setError(err instanceof Error ? err.message : 'Error'),
  });

  function handleFileChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    setFilename(file.name);
    setSelectedFile(file);
    if (!file.name.toLowerCase().endsWith('.pdf')) {
      const reader = new FileReader();
      reader.onload = () => setText(reader.result as string);
      reader.readAsText(file);
    } else {
      setText('');
    }
  }

  return (
    <div>
      <h1>Resume</h1>

      {versions.length > 0 && (
        <section className="card resumeCurrent">
          <p className="eyebrow">Version history</p>
          <div className="resumeVersionList">
            {versions.map((v) => (
              <div key={v.id}>
                <div className={`resumeVersionItem${v.isActive ? ' active' : ''}`}>
                  <span>
                    v{v.version} — {v.filename} —{' '}
                    {new Date(v.uploadedAt).toLocaleDateString()}
                  </span>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    {v.isActive && <span className="activeBadge">Active</span>}
                    <button
                      style={{ padding: '6px 12px', fontSize: 12 }}
                      onClick={() => setExpandedId(expandedId === v.id ? null : v.id)}
                    >
                      {expandedId === v.id ? 'Hide text' : 'View text'}
                    </button>
                    {!v.isActive && (
                      <button
                        style={{ padding: '6px 12px', fontSize: 12 }}
                        onClick={() => activateMutation.mutate(v.id)}
                        disabled={activateMutation.isPending && activateMutation.variables === v.id}
                      >
                        {activateMutation.isPending && activateMutation.variables === v.id ? 'Setting…' : 'Set Active'}
                      </button>
                    )}
                  </div>
                </div>
                {expandedId === v.id && (
                  <pre style={{
                    margin: '8px 0 0',
                    padding: '12px',
                    background: 'var(--surface)',
                    borderRadius: 6,
                    fontSize: 12,
                    lineHeight: 1.6,
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-word',
                    maxHeight: 400,
                    overflowY: 'auto',
                    color: 'var(--text)',
                  }}>
                    {v.rawText}
                  </pre>
                )}
              </div>
            ))}
          </div>
        </section>
      )}

      <section className="card form">
        <h2>Upload new version</h2>
        <form onSubmit={(e) => { e.preventDefault(); uploadMutation.mutate(); }}>
          <label>
            Upload .txt file <span className="muted">(or paste text below)</span>
            <input ref={fileRef} type="file" accept=".pdf,.txt,.md" onChange={handleFileChange} />
          </label>
          <label>
            Resume text
            <textarea
              value={text}
              onChange={(e) => setText(e.target.value)}
              rows={12}
              placeholder="Paste your CV text here…"
            />
          </label>
          {error && <p className="error">{error}</p>}
          <button type="submit" disabled={uploadMutation.isPending || (!selectedFile && !text.trim())}>
            {uploadMutation.isPending ? 'Uploading…' : 'Save Resume'}
          </button>
        </form>
      </section>
    </div>
  );
}
