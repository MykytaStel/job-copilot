import { useRef } from 'react';
import toast from 'react-hot-toast';
import { cleanupExtractedResumeText, extractPdfText } from './profile.utils';

export function useResumePicker(setRawText: (value: string) => void) {
  const fileInputRef = useRef<HTMLInputElement>(null);

  async function handleFileChange(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) return;
    event.target.value = '';

    if (file.type === 'application/pdf') {
      const loadingToast = toast.loading('Читаємо PDF…');
      try {
        const text = await extractPdfText(file);
        if (text.trim()) {
          setRawText(text);
          toast.success(`PDF завантажено: ${file.name}`, { id: loadingToast });
        } else {
          toast.error('PDF порожній або захищений — спробуйте .txt', { id: loadingToast });
        }
      } catch {
        toast.error('Не вдалося прочитати PDF', { id: loadingToast });
      }
      return;
    }

    const reader = new FileReader();
    reader.onload = (loadEvent) => {
      const text = loadEvent.target?.result;
      const cleanedText = typeof text === 'string' ? cleanupExtractedResumeText(text) : '';
      if (cleanedText.trim()) {
        setRawText(cleanedText);
        toast.success(`Файл завантажено: ${file.name}`);
      } else {
        toast.error('Файл порожній або не вдалося прочитати');
      }
    };
    reader.onerror = () => toast.error('Помилка читання файлу');
    reader.readAsText(file, 'UTF-8');
  }

  return {
    fileInputRef,
    openFilePicker: () => fileInputRef.current?.click(),
    handleFileChange,
  };
}
