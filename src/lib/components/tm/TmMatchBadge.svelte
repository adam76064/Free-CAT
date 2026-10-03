<script lang="ts">
  // TmMatchBadge — displays a colour-coded % badge for TM matches.
  import { matchColor, matchLabel } from '$lib/stores/tm';

  interface Props { pct: number; }
  let { pct }: Props = $props();
</script>

<span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-semibold tabular-nums {matchColor(pct)}">
  {matchLabel(pct)}
</span>
