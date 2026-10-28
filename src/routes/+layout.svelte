<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { browser } from '$app/environment';
  import { settings, applyTheme } from '$lib/stores/settings';
  import Icon from '$lib/components/ui/Icon.svelte';
  import ToastContainer from '$lib/components/ui/ToastContainer.svelte';
  import KeyboardShortcutsModal from '$lib/components/ui/KeyboardShortcutsModal.svelte';

  let showShortcuts = false;

  // Theme is applied by the settings store at startup; here we only follow
  // OS changes while the user's choice is 'system'.
  onMount(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = () => {
      if (get(settings).theme === 'system') applyTheme('system');
    };
    mq.addEventListener('change', onChange);

    const onKeydown = (e: KeyboardEvent) => {
      // Ctrl+/ or Cmd+/ to toggle keyboard shortcuts modal
      if ((e.ctrlKey || e.metaKey) && e.key === '/') {
        e.preventDefault();
        showShortcuts = !showShortcuts;
      }
      // Ctrl+Shift+D to cycle dark mode
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === 'D' || e.key === 'd')) {
        e.preventDefault();
        cycleTheme();
      }
    };
    window.addEventListener('keydown', onKeydown);

    return () => {
      mq.removeEventListener('change', onChange);
      window.removeEventListener('keydown', onKeydown);
    };
  });

  function cycleTheme() {
    const cur = $settings.theme;
    const next = cur === 'dark' ? 'light' : cur === 'light' ? 'system' : 'dark';
    settings.set({ ...$settings, theme: next });
  }

  $: pathname = $page.url.pathname;

  const NAV = [
    { href: '/',        match: (p: string) => p === '/',                 label: 'Projects', icon: 'home',       title: 'Projects Dashboard' },
    { href: '/tm',      match: (p: string) => p.startsWith('/tm'),       label: 'TM',       icon: 'brain',      title: 'Translation Memory' },
    { href: '/tb',      match: (p: string) => p.startsWith('/tb'),       label: 'Terms',    icon: 'book-open',  title: 'Termbase Manager' },
    { href: '/aligner', match: (p: string) => p.startsWith('/aligner'), label: 'Align',    icon: 'align-left', title: 'Bilingual Aligner' },
  ] as const;
</script>

<!-- Root layout: sidebar nav + main content area -->
<div class="flex h-screen w-screen overflow-hidden bg-gray-50 dark:bg-gray-950">

  <!-- Sidebar Navigation (64px wide, icon + label) -->
  <nav
    class="flex flex-col items-center w-16 bg-gray-900 dark:bg-gray-950 border-r border-gray-800 py-3 gap-1 shrink-0 select-none"
    aria-label="Main Navigation"
  >
    <!-- Logo mark -->
    <div class="w-9 h-9 bg-brand-500 rounded-xl flex items-center justify-center mb-4 shadow-lg shadow-brand-900/50 shrink-0">
      <span class="text-white text-[11px] font-black tracking-tighter select-none">FC</span>
    </div>

    <!-- Primary nav items -->
    {#each NAV as item}
      <a
        href={item.href}
        class="w-12 flex flex-col items-center py-2 px-1 rounded-xl gap-0.5 transition-colors select-none
               {item.match(pathname)
                 ? 'bg-brand-500/20 text-brand-400'
                 : 'text-gray-500 hover:text-gray-200 hover:bg-gray-800'}"
        title={item.title}
      >
        <Icon name={item.icon} size={18} />
        <span class="text-[9px] font-semibold leading-none">{item.label}</span>
      </a>
    {/each}

    <!-- Spacer -->
    <div class="flex-1"></div>

    <!-- Shortcuts Modal Button -->
    <button
      type="button"
      onclick={() => (showShortcuts = true)}
      class="w-12 flex flex-col items-center py-1.5 px-1 rounded-xl gap-0.5 text-gray-500 hover:text-gray-200 hover:bg-gray-800 transition-colors cursor-pointer"
      title="Keyboard Shortcuts (Ctrl+/)"
    >
      <Icon name="keyboard" size={17} />
      <span class="text-[8px] font-medium leading-none text-gray-400">Keys</span>
    </button>

    <!-- Theme quick toggle -->
    <button
      type="button"
      onclick={cycleTheme}
      class="w-12 flex flex-col items-center py-1.5 px-1 rounded-xl gap-0.5 text-gray-500 hover:text-gray-200 hover:bg-gray-800 transition-colors cursor-pointer"
      title="Theme: {$settings.theme} (Click to cycle, or Ctrl+Shift+D)"
    >
      <Icon name={$settings.theme === 'dark' ? 'moon' : 'sun'} size={17} />
      <span class="text-[8px] font-medium capitalize leading-none text-gray-400">{$settings.theme}</span>
    </button>

    <!-- Settings (pinned to bottom) -->
    <a
      href="/settings"
      class="w-12 flex flex-col items-center py-2 px-1 rounded-xl gap-0.5 transition-colors select-none
             {pathname.startsWith('/settings')
               ? 'bg-brand-500/20 text-brand-400'
               : 'text-gray-500 hover:text-gray-200 hover:bg-gray-800'}"
      title="Settings"
    >
      <Icon name="cog" size={18} />
      <span class="text-[9px] font-semibold leading-none">Settings</span>
    </a>
  </nav>

  <!-- Main content -->
  <main class="flex-1 overflow-hidden">
    <slot />
  </main>

</div>

<!-- Keyboard shortcuts cheat-sheet modal -->
<KeyboardShortcutsModal open={showShortcuts} onClose={() => (showShortcuts = false)} />

<!-- Global toast notifications (rendered above everything) -->
<ToastContainer />
