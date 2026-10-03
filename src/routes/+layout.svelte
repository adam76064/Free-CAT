<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';

  // Dark mode: check system preference + stored preference
  import { onMount } from 'svelte';

  onMount(() => {
    const stored = localStorage.getItem('theme');
    const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (stored === 'dark' || (!stored && systemDark)) {
      document.documentElement.classList.add('dark');
    }
  });
</script>

<!-- Root layout: sidebar nav + main content area -->
<div class="flex h-screen w-screen overflow-hidden bg-gray-50 dark:bg-gray-950">

  <!-- Icon Sidebar (48px) -->
  <nav class="flex flex-col items-center w-12 bg-gray-900 dark:bg-gray-950 border-r border-gray-800 py-3 gap-1 shrink-0">

    <!-- Logo -->
    <div class="w-8 h-8 bg-brand-500 rounded-lg flex items-center justify-center mb-3">
      <span class="text-white text-xs font-bold">FC</span>
    </div>

    <!-- Nav icons -->
    <NavIcon href="/" title="Home" icon="🏠" active={$page.url.pathname === '/'} />
    <NavIcon href="/tm" title="Translation Memory" icon="🧠" active={$page.url.pathname.startsWith('/tm')} />
    <NavIcon href="/tb" title="Termbase" icon="📖" active={$page.url.pathname.startsWith('/tb')} />
    <NavIcon href="/settings" title="Settings" icon="⚙️" active={$page.url.pathname.startsWith('/settings')} />
  </nav>

  <!-- Main content -->
  <main class="flex-1 overflow-hidden">
    <slot />
  </main>

</div>

<!-- Nav Icon component (inline for layout file) -->
<script lang="ts" context="module">
</script>

{#snippet NavIcon({ href, title, icon, active }: { href: string; title: string; icon: string; active: boolean })}
  <a
    {href}
    class="w-9 h-9 rounded-lg flex items-center justify-center text-lg transition-colors
           {active
             ? 'bg-brand-500/20 text-brand-400'
             : 'text-gray-500 hover:text-gray-300 hover:bg-gray-800'}"
    title={title}
  >
    {icon}
  </a>
{/snippet}
