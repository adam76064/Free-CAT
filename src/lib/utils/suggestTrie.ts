/**
 * SuggestionTrie — Fast in-memory Trie for AutoSuggest and predictive typing.
 * Indexes terms from active Termbases, TM target phrases, and AutoText.
 */

export interface SuggestionItem {
  text: string;
  source: 'termbase' | 'tm' | 'autotext';
  matchScore?: number;
  displayText?: string;
}

interface TrieNode {
  children: Map<string, TrieNode>;
  items: SuggestionItem[];
  isEndOfWord: boolean;
}

export class SuggestionTrie {
  private root: TrieNode;

  constructor() {
    this.root = this.createNode();
  }

  private createNode(): TrieNode {
    return {
      children: new Map(),
      items: [],
      isEndOfWord: false
    };
  }

  /** Clear all indexed suggestions */
  public clear(): void {
    this.root = this.createNode();
  }

  /** Insert a word or multi-word phrase into the trie */
  public insert(phrase: string, item: SuggestionItem): void {
    const cleaned = phrase.trim().toLowerCase();
    if (!cleaned) return;

    let current = this.root;
    for (const char of cleaned) {
      if (!current.children.has(char)) {
        current.children.set(char, this.createNode());
      }
      current = current.children.get(char)!;
    }

    current.isEndOfWord = true;
    // Avoid duplicate insertions
    if (!current.items.some(existing => existing.text.toLowerCase() === item.text.toLowerCase())) {
      current.items.push(item);
    }
  }

  /**
   * Search for phrases matching a given prefix.
   * Returns top matching suggestions sorted by source priority (termbase > tm > autotext) and matchScore.
   */
  public searchPrefix(prefix: string, maxResults = 5): SuggestionItem[] {
    const cleaned = prefix.trim().toLowerCase();
    if (!cleaned || cleaned.length < 2) return [];

    let current = this.root;
    for (const char of cleaned) {
      if (!current.children.has(char)) {
        return [];
      }
      current = current.children.get(char)!;
    }

    const results: SuggestionItem[] = [];
    this.collectAll(current, results, maxResults * 3);

    // Sort by priority: termbase first, then TM matches
    results.sort((a, b) => {
      const priorityOrder = { termbase: 0, tm: 1, autotext: 2 };
      const pA = priorityOrder[a.source] ?? 3;
      const pB = priorityOrder[b.source] ?? 3;
      if (pA !== pB) return pA - pB;
      return (b.matchScore ?? 0) - (a.matchScore ?? 0);
    });

    return results.slice(0, maxResults);
  }

  private collectAll(node: TrieNode, results: SuggestionItem[], limit: number): void {
    if (results.length >= limit) return;

    if (node.isEndOfWord) {
      for (const item of node.items) {
        if (!results.some(r => r.text.toLowerCase() === item.text.toLowerCase())) {
          results.push(item);
          if (results.length >= limit) return;
        }
      }
    }

    for (const child of node.children.values()) {
      this.collectAll(child, results, limit);
      if (results.length >= limit) return;
    }
  }
}
