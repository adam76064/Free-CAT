// src/lib/utils/virtualWindow.ts
// Windowing math for variable-height lists (the segment grid).
//
// Segment rows vary in height (wrapped text, Arabic/CJK, tags, comments), so a
// fixed row-height formula can't work. HeightIndex stores one height per row
// in a Fenwick tree, giving O(log n) "offset of row i", "row at pixel y" and
// "update one row's height" — fast enough to run on every scroll event even
// with 100k+ segments. Unmeasured rows use an estimate until they render.

export class HeightIndex {
  private n = 0;
  private tree = new Float64Array(1); // 1-indexed Fenwick tree
  private h = new Float64Array(0);

  constructor(public estimate = 76) {}

  get length(): number {
    return this.n;
  }

  /** Rebuild from scratch in O(n). */
  reset(heights: ArrayLike<number>): void {
    const n = heights.length;
    this.n = n;
    this.h = new Float64Array(n);
    this.tree = new Float64Array(n + 1);
    for (let i = 0; i < n; i++) {
      this.h[i] = heights[i];
      this.tree[i + 1] = heights[i];
    }
    for (let i = 1; i <= n; i++) {
      const j = i + (i & -i);
      if (j <= n) this.tree[j] += this.tree[i];
    }
  }

  get(i: number): number {
    return this.h[i] ?? this.estimate;
  }

  /** Set one row's height. Returns true if the value actually changed. */
  set(i: number, height: number): boolean {
    if (i < 0 || i >= this.n) return false;
    const delta = height - this.h[i];
    if (Math.abs(delta) < 0.5) return false;
    this.h[i] = height;
    for (let k = i + 1; k <= this.n; k += k & -k) this.tree[k] += delta;
    return true;
  }

  /** Pixel offset of the top of row i (sum of heights of rows 0..i-1). */
  offset(i: number): number {
    let k = Math.max(0, Math.min(i, this.n));
    let sum = 0;
    while (k > 0) {
      sum += this.tree[k];
      k -= k & -k;
    }
    return sum;
  }

  total(): number {
    return this.offset(this.n);
  }

  /** Index of the row containing pixel y (clamped to [0, n-1]). */
  indexAt(y: number): number {
    if (this.n === 0) return 0;
    let pos = 0;
    let step = 1;
    while (step * 2 <= this.n) step *= 2;
    for (; step > 0; step >>= 1) {
      const next = pos + step;
      if (next <= this.n && this.tree[next] <= y) {
        pos = next;
        y -= this.tree[next];
      }
    }
    return Math.min(pos, this.n - 1);
  }
}

export interface WindowRange {
  start: number; // first rendered row (inclusive)
  end: number;   // last rendered row (exclusive)
  topPad: number;    // spacer height above the rendered rows
  bottomPad: number; // spacer height below the rendered rows
}

/**
 * Rows to render for a viewport. `scrollTop` is relative to the top of the
 * list (not the scroll container). `overscanPx` renders extra rows above and
 * below so fast scrolling doesn't flash blank space.
 */
export function computeWindow(
  idx: HeightIndex,
  scrollTop: number,
  viewportHeight: number,
  overscanPx = 600,
): WindowRange {
  const n = idx.length;
  if (n === 0) return { start: 0, end: 0, topPad: 0, bottomPad: 0 };
  const top = Math.max(0, scrollTop - overscanPx);
  const bottom = Math.max(0, scrollTop) + viewportHeight + overscanPx;
  const start = idx.indexAt(top);
  const end = Math.min(n, idx.indexAt(bottom) + 1);
  const topPad = idx.offset(start);
  const bottomPad = Math.max(0, idx.total() - idx.offset(end));
  return { start, end, topPad, bottomPad };
}

/** Svelte action: report an element's border-box height whenever it changes. */
export function observeHeight(node: HTMLElement, cb: (height: number) => void) {
  let callback = cb;
  const ro = new ResizeObserver((entries) => {
    const box = entries[0]?.borderBoxSize?.[0];
    callback(box ? box.blockSize : node.offsetHeight);
  });
  ro.observe(node);
  return {
    update(next: (height: number) => void) {
      callback = next;
    },
    destroy() {
      ro.disconnect();
    },
  };
}

/**
 * Stateful controller for a windowed list of items with unique ids.
 * Owns the HeightIndex plus the measured-height cache, so a component only
 * has to (1) call `sync(items)` when its list changes, (2) report row heights
 * via `setRowHeight`, and (3) ask `window(...)` which rows to render.
 */
export class WindowedList {
  private heights: HeightIndex;
  private measured = new Map<string, number>(); // id -> last measured height
  private idToIndex = new Map<string, number>();
  private lastIds: string[] = [];

  constructor(private rowEstimate = 76, private overscanPx = 800) {
    this.heights = new HeightIndex(rowEstimate);
  }

  get length(): number {
    return this.heights.length;
  }

  /**
   * Call whenever the item list may have changed. If the ids and their order
   * are unchanged (e.g. a keystroke replaced the array) this is O(n) compares
   * and no rebuild. Returns true if the height index was rebuilt.
   */
  sync(items: ReadonlyArray<{ id: string }>): boolean {
    const n = items.length;
    let same = n === this.lastIds.length;
    for (let i = 0; same && i < n; i++) if (items[i].id !== this.lastIds[i]) same = false;
    if (same) return false;

    let sum = 0;
    let count = 0;
    for (let i = 0; i < n; i++) {
      const m = this.measured.get(items[i].id);
      if (m !== undefined) {
        sum += m;
        count++;
      }
    }
    // Once enough rows are measured, use their average for unmeasured rows.
    const estimate = count >= 20 ? sum / count : this.rowEstimate;
    const heights = new Float64Array(n);
    const ids = new Map<string, number>();
    const lastIds = new Array<string>(n);
    for (let i = 0; i < n; i++) {
      const id = items[i].id;
      ids.set(id, i);
      lastIds[i] = id;
      heights[i] = this.measured.get(id) ?? estimate;
    }
    this.idToIndex = ids;
    this.lastIds = lastIds;
    this.heights.estimate = estimate;
    this.heights.reset(heights);
    if (this.measured.size > n * 2 + 1000) {
      this.measured = new Map([...this.measured].filter(([id]) => ids.has(id)));
    }
    return true;
  }

  /** Record a measured row height. Returns true if the layout changed. */
  setRowHeight(id: string, height: number): boolean {
    this.measured.set(id, height);
    const i = this.idToIndex.get(id);
    return i !== undefined && this.heights.set(i, height);
  }

  offsetOf(index: number): number {
    return this.heights.offset(index);
  }

  totalHeight(): number {
    return this.heights.total();
  }

  /** Rows to render; `scrollTop` is relative to the top of the list. */
  window(scrollTop: number, viewportHeight: number): WindowRange {
    return computeWindow(this.heights, scrollTop, viewportHeight, this.overscanPx);
  }
}
