import { writable } from 'svelte/store';

export interface AutoTextEntry {
  id: string;
  trigger: string;
  expansion: string;
  description?: string;
}

const STORAGE_KEY = 'freecat_autotext_entries';

const DEFAULT_ENTRIES: AutoTextEntry[] = [
  { id: '1', trigger: 'w/o', expansion: 'without', description: 'Standard preposition shorthand' },
  { id: '2', trigger: 'asap', expansion: 'as soon as possible', description: 'Urgency shorthand' },
  { id: '3', trigger: 't&c', expansion: 'terms and conditions', description: 'Legal boilerplate' },
  { id: '4', trigger: 'fwiw', expansion: 'for what it\'s worth', description: 'Commentary phrase' },
  { id: '5', trigger: 'i.e.', expansion: 'that is', description: 'Latin id est' },
  { id: '6', trigger: 'e.g.', expansion: 'for example', description: 'Latin exempli gratia' },
  { id: '7', trigger: 'ip', expansion: 'intellectual property', description: 'Legal / patent term' },
  { id: '8', trigger: 'nda', expansion: 'non-disclosure agreement', description: 'Legal contract' },
  { id: '9', trigger: 'pls', expansion: 'please', description: 'Polite request' },
  { id: '10', trigger: 'thx', expansion: 'thank you', description: 'Gratitude' },
  { id: '11', trigger: 'ui', expansion: 'user interface', description: 'Software UI localization' },
  { id: '12', trigger: 'qa', expansion: 'quality assurance', description: 'Testing & verification' },
];

function loadInitialEntries(): AutoTextEntry[] {
  if (typeof window === 'undefined') return DEFAULT_ENTRIES;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) return parsed;
    }
  } catch (e) {
    console.warn('Failed to load autotext entries from localStorage:', e);
  }
  return DEFAULT_ENTRIES;
}

export const autoTextEntries = writable<AutoTextEntry[]>(loadInitialEntries());

autoTextEntries.subscribe((entries) => {
  if (typeof window === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
  } catch (e) {
    console.warn('Failed to persist autotext entries to localStorage:', e);
  }
});

export function addAutoTextEntry(trigger: string, expansion: string, description = '') {
  const cleanTrigger = trigger.trim().toLowerCase();
  const cleanExpansion = expansion.trim();
  if (!cleanTrigger || !cleanExpansion) return;

  autoTextEntries.update(list => {
    const existing = list.findIndex(e => e.trigger.toLowerCase() === cleanTrigger);
    if (existing >= 0) {
      list[existing] = { ...list[existing], expansion: cleanExpansion, description: description.trim() || list[existing].description };
      return [...list];
    }
    return [
      ...list,
      {
        id: `at_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
        trigger: cleanTrigger,
        expansion: cleanExpansion,
        description: description.trim()
      }
    ];
  });
}

export function deleteAutoTextEntry(id: string) {
  autoTextEntries.update(list => list.filter(e => e.id !== id));
}

export function resetAutoTextEntries() {
  autoTextEntries.set([...DEFAULT_ENTRIES]);
}
