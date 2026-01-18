<script lang="ts">
  import type { CompanyProfile, Subsidiary, CorporateAction, NewsItem } from './StockAnalysis.types';

  let {
    profile,
    subsidiaries = [],
    corporateActions = [],
    news = [],
  }: {
    profile: CompanyProfile | null;
    subsidiaries: Subsidiary[];
    corporateActions: CorporateAction[];
    news: NewsItem[];
  } = $props();

  type TimelineItem =
    | { type: 'action'; date: Date; data: CorporateAction }
    | { type: 'news'; date: Date; data: NewsItem };

  let timeline = $derived(() => {
    const items: TimelineItem[] = [
      ...corporateActions.map((ca) => ({
        type: 'action' as const,
        date: new Date(ca.effective_date ?? ca.announced_date),
        data: ca,
      })),
      ...news.map((n) => ({
        type: 'news' as const,
        date: new Date(n.published_at),
        data: n,
      })),
    ];
    return items.sort((a, b) => b.date.getTime() - a.date.getTime());
  });

  function formatDate(date: Date): string {
    return date.toLocaleDateString('id-ID', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
    });
  }

  function getActionColor(type: string): string {
    const colors: Record<string, string> = {
      dividend: 'variant-filled-success',
      stock_split: 'variant-filled-primary',
      rights_issue: 'variant-filled-warning',
      acquisition: 'variant-filled-secondary',
    };
    return colors[type] ?? 'variant-filled-surface';
  }
</script>

<div class="space-y-6">
  {#if profile}
    <div class="card p-4">
      <h3 class="h4 mb-3">About {profile.name}</h3>
      <p class="text-sm text-surface-600-300-token">
        {profile.business_summary ?? profile.description ?? 'No description available'}
      </p>

      {#if profile.website}
        <a
          href={profile.website}
          target="_blank"
          rel="noopener noreferrer"
          class="text-sm text-primary-500 hover:underline mt-2 inline-block"
        >
          Visit Website
        </a>
      {/if}
    </div>
  {/if}

  {#if subsidiaries.length > 0}
    <div class="card p-4">
      <h3 class="h4 mb-3">Business Units & Subsidiaries</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        {#each subsidiaries as sub}
          <div class="p-3 bg-surface-100-800-token rounded-lg">
            <div class="flex justify-between items-start">
              <span class="font-medium">{sub.name}</span>
              <span class="badge variant-soft-primary">{sub.ownership_percent}%</span>
            </div>
            {#if sub.business_type}
              <span class="text-xs text-surface-500">{sub.business_type}</span>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}

  {#if timeline().length > 0}
    <div class="card p-4">
      <h3 class="h4 mb-3">Timeline & News</h3>
      <div class="relative border-l-2 border-surface-300-600-token pl-4 space-y-4">
        {#each timeline() as item}
          <div class="relative">
            <div
              class="absolute -left-[1.4rem] w-3 h-3 rounded-full {item.type === 'action'
                ? 'bg-primary-500'
                : 'bg-surface-400'}"
            ></div>

            {#if item.type === 'action'}
              {@const action = item.data}
              <div
                class="p-3 bg-primary-50 dark:bg-primary-900/20 rounded-lg border-l-4 border-primary-500"
              >
                <div class="flex justify-between items-start">
                  <div>
                    <span class="badge {getActionColor(action.action_type)} text-xs">
                      {action.action_type}
                    </span>
                    <h4 class="font-medium mt-1">{action.description}</h4>
                  </div>
                  <span class="text-xs text-surface-500">{formatDate(item.date)}</span>
                </div>
                {#if action.value}
                  <span class="text-sm font-mono text-primary-700 dark:text-primary-300">
                    Rp {action.value.toLocaleString('id-ID')}
                  </span>
                {/if}
              </div>
            {:else}
              {@const n = item.data}
              <div class="p-3 bg-surface-100-800-token rounded-lg">
                <div class="flex justify-between items-start gap-2">
                  <a
                    href={n.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="font-medium hover:text-primary-500 hover:underline"
                  >
                    {n.title}
                  </a>
                  <span class="text-xs text-surface-500 whitespace-nowrap">
                    {formatDate(item.date)}
                  </span>
                </div>
                <div class="flex items-center gap-2 mt-1">
                  <span class="text-xs text-surface-400">{n.source}</span>
                  {#each n.keywords as kw}
                    <span class="badge variant-soft text-xs">{kw}</span>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="card p-4">
      <p class="text-sm text-surface-500 text-center">No business updates available</p>
    </div>
  {/if}
</div>
