// src/lib/stores/tts.ts — text-to-speech for acoustic proofreading.
// Wraps the browser SpeechSynthesis API; state is exposed as a store so any
// component can show play/continuous/rate controls.
import { writable, get } from 'svelte/store';

export interface TtsState {
  playing: boolean;
  /** Keep reading the next segment automatically when one finishes. */
  continuous: boolean;
  rate: number;
}

const RATES = [1.0, 1.2, 1.5];

function createTtsStore() {
  const store = writable<TtsState>({ playing: false, continuous: false, rate: 1.0 });
  const { subscribe, update } = store;

  const supported = () => typeof window !== 'undefined' && 'speechSynthesis' in window;

  function stop() {
    if (supported()) window.speechSynthesis.cancel();
    update((s) => ({ ...s, playing: false }));
  }

  function speak(text: string, lang: string, onEnd?: () => void) {
    if (!supported()) return;
    window.speechSynthesis.cancel();
    if (!text.trim()) {
      update((s) => ({ ...s, playing: false }));
      return;
    }

    const utter = new SpeechSynthesisUtterance(text);
    utter.rate = get(store).rate;
    utter.lang = lang || 'en';

    try {
      const voices = window.speechSynthesis.getVoices();
      const prefix = (lang || 'en').toLowerCase().slice(0, 2);
      const matched = voices.find((v) => v.lang.toLowerCase().startsWith(prefix));
      if (matched) utter.voice = matched;
    } catch {
      /* voice selection is best-effort */
    }

    utter.onend = () => {
      update((s) => ({ ...s, playing: false }));
      onEnd?.();
    };
    utter.onerror = () => update((s) => ({ ...s, playing: false }));

    update((s) => ({ ...s, playing: true }));
    window.speechSynthesis.speak(utter);
  }

  return {
    subscribe,
    stop,
    speak,
    toggleContinuous: () => update((s) => ({ ...s, continuous: !s.continuous })),
    /** 1.0x → 1.2x → 1.5x → 1.0x */
    cycleRate: () =>
      update((s) => ({ ...s, rate: RATES[(RATES.indexOf(s.rate) + 1) % RATES.length] })),
  };
}

export const tts = createTtsStore();
