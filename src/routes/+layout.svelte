<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import ToastContainer from '$lib/components/ui/ToastContainer.svelte';

  onMount(() => {
    const stored = localStorage.getItem('theme');
    const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (stored === 'dark' || (!stored && systemDark)) {
      document.documentElement.classList.add('dark');
    }
  });

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
    class="flex flex-col items-center w-16 bg-gray-900 dark:bg-gray-950 border-r border-gray-800 py-3 gap-1 shrink-0"
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

<!-- Global toast notifications (rendered above everything) -->
<ToastContainer />
