import { useRef } from 'react';

import { useToast } from '../../context/ToastContext';
import { cleanupExtractedResumeText, extractPdfText } from './profile.utils';

export function useResumePicker(setRawText: (value: string) => void) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { showToast, dismissToast } = useToast();

  async function handleFileChange(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) return;
    event.target.value = '';

    const MAX_SIZE_BYTES = 5 * 1024 * 1024;
    if (file.size > MAX_SIZE_BYTES) {
      showToast({
        type: 'error',
        message: `Файл задто великий (${(file.size / 1024 / 1024).toFixed(1)} МБ). Максимальний розмір — 5 МБ.`,
      });
      return;
    }

    const ext = file.name.split('.').pop()?.toLowerCase();
    const allowedExts = new Set(['pdf', 'txt', 'md', 'text']);
    if (!allowedExts.has(ext ?? '')) {
      showToast({ type: 'error', message: 'Непідтримуваний формат. Підтримуються: PDF, TXT, MD.' });
      return;
    }

    if (file.type === 'application/pdf') {
      const loadingId = showToast({ type: 'info', message: 'Читаємо PDF…', durationMs: 60_000 });
      try {
        const text = await extractPdfText(file);
        dismissToast(loadingId);
        if (text.trim()) {
          setRawText(text);
          showToast({ type: 'success', message: `PDF завантажено: ${file.name}` });
        } else {
          showToast({ type: 'error', message: 'PDF порожній або захищений — спробуйте .txt' });
        }
      } catch {
        dismissToast(loadingId);
        showToast({ type: 'error', message: 'Не вдалося прочитати PDF' });
      }
      return;
    }

    const reader = new FileReader();
    reader.onload = (loadEvent) => {
      const text = loadEvent.target?.result;
      const cleanedText = typeof text === 'string' ? cleanupExtractedResumeText(text) : '';
      if (cleanedText.trim()) {
        setRawText(cleanedText);
        showToast({ type: 'success', message: `Файл завантажено: ${file.name}` });
      } else {
        showToast({ type: 'error', message: 'Файл порожній або не вдалося прочитати' });
      }
    };
    reader.onerror = () => showToast({ type: 'error', message: 'Помилка читання файлу' });
    reader.readAsText(file, 'UTF-8');
  }

  return {
    fileInputRef,
    openFilePicker: () => fileInputRef.current?.click(),
    handleFileChange,
  };
}
