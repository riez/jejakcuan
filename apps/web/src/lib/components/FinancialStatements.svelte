<script lang="ts">
  interface IncomeStatement {
    symbol: string;
    fiscal_year: number;
    fiscal_quarter: number | null;
    period_end: string;
    revenue: number | null;
    gross_profit: number | null;
    operating_income: number | null;
    net_income: number | null;
    eps: number | null;
    gross_margin: number | null;
    operating_margin: number | null;
    net_margin: number | null;
  }

  interface BalanceSheet {
    symbol: string;
    fiscal_year: number;
    fiscal_quarter: number | null;
    period_end: string;
    total_assets: number | null;
    total_liabilities: number | null;
    total_equity: number | null;
    total_debt: number | null;
    current_ratio: number | null;
    debt_to_equity: number | null;
  }

  interface CashFlow {
    symbol: string;
    fiscal_year: number;
    fiscal_quarter: number | null;
    period_end: string;
    operating_cash_flow: number | null;
    capital_expenditure: number | null;
    free_cash_flow: number | null;
  }

  interface FinancialRatios {
    symbol: string;
    fiscal_year: number;
    fiscal_quarter: number | null;
    period_end: string;
    roe: number | null;
    roa: number | null;
    gross_margin: number | null;
    operating_margin: number | null;
    net_margin: number | null;
    current_ratio: number | null;
    debt_to_equity: number | null;
    eps: number | null;
    revenue_growth: number | null;
    earnings_growth: number | null;
  }

  let {
    incomeStatements = [],
    balanceSheets = [],
    cashFlows = [],
    ratios = [],
  }: {
    incomeStatements: IncomeStatement[];
    balanceSheets: BalanceSheet[];
    cashFlows: CashFlow[];
    ratios: FinancialRatios[];
  } = $props();

  let activeTab: 'income' | 'balance' | 'cashflow' | 'ratios' = $state('income');

  function formatBillions(value: number | null): string {
    if (value === null) return '-';
    const billions = value / 1_000_000_000;
    if (billions >= 1000) {
      return `${(billions / 1000).toFixed(1)}T`;
    }
    if (billions >= 1) {
      return `${billions.toFixed(1)}B`;
    }
    const millions = value / 1_000_000;
    return `${millions.toFixed(0)}M`;
  }

  function formatPercent(value: number | null): string {
    if (value === null) return '-';
    return `${(value * 100).toFixed(1)}%`;
  }

  function formatRatio(value: number | null): string {
    if (value === null) return '-';
    return value.toFixed(2);
  }

  function getGrowthColor(current: number | null, previous: number | null): string {
    if (current === null || previous === null || previous === 0) return '';
    const growth = (current - previous) / Math.abs(previous);
    if (growth > 0.1) return 'text-success-500';
    if (growth < -0.1) return 'text-error-500';
    return '';
  }

  function getYearLabel(year: number, quarter: number | null): string {
    if (quarter) return `Q${quarter} ${year}`;
    return `FY${year}`;
  }
</script>

<div class="card">
  <header class="card-header flex gap-2 overflow-x-auto pb-0">
    <button
      class="btn btn-sm {activeTab === 'income' ? 'variant-filled-primary' : 'variant-ghost'}"
      onclick={() => (activeTab = 'income')}
    >
      Income Statement
    </button>
    <button
      class="btn btn-sm {activeTab === 'balance' ? 'variant-filled-primary' : 'variant-ghost'}"
      onclick={() => (activeTab = 'balance')}
    >
      Balance Sheet
    </button>
    <button
      class="btn btn-sm {activeTab === 'cashflow' ? 'variant-filled-primary' : 'variant-ghost'}"
      onclick={() => (activeTab = 'cashflow')}
    >
      Cash Flow
    </button>
    <button
      class="btn btn-sm {activeTab === 'ratios' ? 'variant-filled-primary' : 'variant-ghost'}"
      onclick={() => (activeTab = 'ratios')}
    >
      Key Ratios
    </button>
  </header>

  <section class="p-4 overflow-x-auto">
    {#if activeTab === 'income'}
      {#if incomeStatements.length === 0}
        <p class="text-center text-surface-500">No income statement data available</p>
      {:else}
        <table class="table table-compact w-full">
          <thead>
            <tr>
              <th class="text-left">Metric</th>
              {#each incomeStatements as stmt}
                <th class="text-right">{getYearLabel(stmt.fiscal_year, stmt.fiscal_quarter)}</th>
              {/each}
            </tr>
          </thead>
          <tbody>
            <tr>
              <td class="font-medium">Revenue</td>
              {#each incomeStatements as stmt, i}
                <td
                  class="text-right font-mono {getGrowthColor(
                    stmt.revenue,
                    incomeStatements[i + 1]?.revenue ?? null,
                  )}"
                >
                  {formatBillions(stmt.revenue)}
                </td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Gross Profit</td>
              {#each incomeStatements as stmt}
                <td class="text-right font-mono">{formatBillions(stmt.gross_profit)}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Operating Income</td>
              {#each incomeStatements as stmt}
                <td class="text-right font-mono">{formatBillions(stmt.operating_income)}</td>
              {/each}
            </tr>
            <tr class="font-semibold">
              <td>Net Income</td>
              {#each incomeStatements as stmt, i}
                <td
                  class="text-right font-mono {getGrowthColor(
                    stmt.net_income,
                    incomeStatements[i + 1]?.net_income ?? null,
                  )}"
                >
                  {formatBillions(stmt.net_income)}
                </td>
              {/each}
            </tr>
            <tr class="border-t">
              <td class="font-medium">EPS</td>
              {#each incomeStatements as stmt}
                <td class="text-right font-mono">{stmt.eps?.toFixed(0) ?? '-'}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Gross Margin</td>
              {#each incomeStatements as stmt}
                <td class="text-right font-mono">{formatPercent(stmt.gross_margin)}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Operating Margin</td>
              {#each incomeStatements as stmt}
                <td class="text-right font-mono">{formatPercent(stmt.operating_margin)}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Net Margin</td>
              {#each incomeStatements as stmt}
                <td class="text-right font-mono">{formatPercent(stmt.net_margin)}</td>
              {/each}
            </tr>
          </tbody>
        </table>
      {/if}
    {:else if activeTab === 'balance'}
      {#if balanceSheets.length === 0}
        <p class="text-center text-surface-500">No balance sheet data available</p>
      {:else}
        <table class="table table-compact w-full">
          <thead>
            <tr>
              <th class="text-left">Metric</th>
              {#each balanceSheets as bs}
                <th class="text-right">{getYearLabel(bs.fiscal_year, bs.fiscal_quarter)}</th>
              {/each}
            </tr>
          </thead>
          <tbody>
            <tr class="font-semibold">
              <td>Total Assets</td>
              {#each balanceSheets as bs}
                <td class="text-right font-mono">{formatBillions(bs.total_assets)}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Total Liabilities</td>
              {#each balanceSheets as bs}
                <td class="text-right font-mono">{formatBillions(bs.total_liabilities)}</td>
              {/each}
            </tr>
            <tr class="font-semibold">
              <td>Total Equity</td>
              {#each balanceSheets as bs, i}
                <td
                  class="text-right font-mono {getGrowthColor(
                    bs.total_equity,
                    balanceSheets[i + 1]?.total_equity ?? null,
                  )}"
                >
                  {formatBillions(bs.total_equity)}
                </td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Total Debt</td>
              {#each balanceSheets as bs}
                <td class="text-right font-mono">{formatBillions(bs.total_debt)}</td>
              {/each}
            </tr>
            <tr class="border-t">
              <td class="font-medium">Current Ratio</td>
              {#each balanceSheets as bs}
                <td class="text-right font-mono">{formatRatio(bs.current_ratio)}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Debt/Equity</td>
              {#each balanceSheets as bs}
                <td class="text-right font-mono">{formatRatio(bs.debt_to_equity)}</td>
              {/each}
            </tr>
          </tbody>
        </table>
      {/if}
    {:else if activeTab === 'cashflow'}
      {#if cashFlows.length === 0}
        <p class="text-center text-surface-500">No cash flow data available</p>
      {:else}
        <table class="table table-compact w-full">
          <thead>
            <tr>
              <th class="text-left">Metric</th>
              {#each cashFlows as cf}
                <th class="text-right">{getYearLabel(cf.fiscal_year, cf.fiscal_quarter)}</th>
              {/each}
            </tr>
          </thead>
          <tbody>
            <tr class="font-semibold">
              <td>Operating Cash Flow</td>
              {#each cashFlows as cf, i}
                <td
                  class="text-right font-mono {getGrowthColor(
                    cf.operating_cash_flow,
                    cashFlows[i + 1]?.operating_cash_flow ?? null,
                  )}"
                >
                  {formatBillions(cf.operating_cash_flow)}
                </td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Capital Expenditure</td>
              {#each cashFlows as cf}
                <td class="text-right font-mono">{formatBillions(cf.capital_expenditure)}</td>
              {/each}
            </tr>
            <tr class="font-semibold border-t">
              <td>Free Cash Flow</td>
              {#each cashFlows as cf, i}
                <td
                  class="text-right font-mono {getGrowthColor(
                    cf.free_cash_flow,
                    cashFlows[i + 1]?.free_cash_flow ?? null,
                  )}"
                >
                  {formatBillions(cf.free_cash_flow)}
                </td>
              {/each}
            </tr>
          </tbody>
        </table>
      {/if}
    {:else if activeTab === 'ratios'}
      {#if ratios.length === 0}
        <p class="text-center text-surface-500">No ratio data available</p>
      {:else}
        <table class="table table-compact w-full">
          <thead>
            <tr>
              <th class="text-left">Ratio</th>
              {#each ratios as r}
                <th class="text-right">{getYearLabel(r.fiscal_year, r.fiscal_quarter)}</th>
              {/each}
            </tr>
          </thead>
          <tbody>
            <tr class="font-semibold">
              <td>ROE</td>
              {#each ratios as r}
                <td class="text-right font-mono">{formatPercent(r.roe)}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">ROA</td>
              {#each ratios as r}
                <td class="text-right font-mono">{formatPercent(r.roa)}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Gross Margin</td>
              {#each ratios as r}
                <td class="text-right font-mono">{formatPercent(r.gross_margin)}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Operating Margin</td>
              {#each ratios as r}
                <td class="text-right font-mono">{formatPercent(r.operating_margin)}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Net Margin</td>
              {#each ratios as r}
                <td class="text-right font-mono">{formatPercent(r.net_margin)}</td>
              {/each}
            </tr>
            <tr class="border-t">
              <td class="font-medium">Current Ratio</td>
              {#each ratios as r}
                <td class="text-right font-mono">{formatRatio(r.current_ratio)}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Debt/Equity</td>
              {#each ratios as r}
                <td class="text-right font-mono">{formatRatio(r.debt_to_equity)}</td>
              {/each}
            </tr>
            <tr class="border-t">
              <td class="font-medium">Revenue Growth</td>
              {#each ratios as r}
                <td class="text-right font-mono">{formatPercent(r.revenue_growth)}</td>
              {/each}
            </tr>
            <tr>
              <td class="font-medium">Earnings Growth</td>
              {#each ratios as r}
                <td class="text-right font-mono">{formatPercent(r.earnings_growth)}</td>
              {/each}
            </tr>
          </tbody>
        </table>
      {/if}
    {/if}
  </section>
</div>
