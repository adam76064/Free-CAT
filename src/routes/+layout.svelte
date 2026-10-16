<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';

  onMount(() => {
    const stored = localStorage.getItem('theme');
    const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (stored === 'dark' || (!stored && systemDark)) {
      document.documentElement.classList.add('dark');
    }
  });

  $: pathname = $page.url.pathname;
</script>

<!-- Root layout: sidebar nav + main content area -->
<div class="flex h-screen w-screen overflow-hidden bg-gray-50 dark:bg-gray-950">

  <!-- Icon Sidebar (48px) -->
  <nav class="flex flex-col items-center w-12 bg-gray-900 dark:bg-gray-950 border-r border-gray-800 py-3 gap-1 shrink-0">

    <!-- Logo mark -->
    <div class="w-8 h-8 bg-brand-500 rounded-lg flex items-center justify-center mb-3">
      <span class="text-white text-xs font-bold">FC</span>
    </div>

    <!-- Home -->
    <a href="/"
       class="w-9 h-9 rounded-lg flex items-center justify-center text-lg transition-colors
              {pathname === '/' ? 'bg-brand-500/20 text-brand-400' : 'text-gray-500 hover:text-gray-300 hover:bg-gray-800'}"
       title="Projects">🏠</a>

    <!-- TM Manager -->
    <a href="/tm"
       class="w-9 h-9 rounded-lg flex items-center justify-center text-lg transition-colors
              {pathname.startsWith('/tm') ? 'bg-brand-500/20 text-brand-400' : 'text-gray-500 hover:text-gray-300 hover:bg-gray-800'}"
       title="Translation Memory">🧠</a>

    <!-- Termbase -->
    <a href="/tb"
       class="w-9 h-9 rounded-lg flex items-center justify-center text-lg transition-colors
              {pathname.startsWith('/tb') ? 'bg-brand-500/20 text-brand-400' : 'text-gray-500 hover:text-gray-300 hover:bg-gray-800'}"
       title="Termbase">📖</a>

    <!-- Aligner -->
    <a href="/aligner"
       class="w-9 h-9 rounded-lg flex items-center justify-center text-lg transition-colors
              {pathname.startsWith('/aligner') ? 'bg-brand-500/20 text-brand-400' : 'text-gray-500 hover:text-gray-300 hover:bg-gray-800'}"
       title="Visual Bilingual Aligner">📐</a>

    <!-- Spacer -->
    <div class="flex-1"></div>

    <!-- Settings (pinned to bottom) -->
    <a href="/settings"
       class="w-9 h-9 rounded-lg flex items-center justify-center text-lg transition-colors
              {pathname.startsWith('/settings') ? 'bg-brand-500/20 text-brand-400' : 'text-gray-500 hover:text-gray-300 hover:bg-gray-800'}"
       title="Settings">⚙️</a>
  </nav>

  <!-- Main content -->
  <main class="flex-1 overflow-hidden">
    <slot />
  </main>

</div>
