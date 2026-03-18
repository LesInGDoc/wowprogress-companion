<script>
  import { onMount, onDestroy } from 'svelte';
  import { Chart, registerables } from 'chart.js';
  import { fetchFilters, fetchPulls } from '../services/api.js';

  Chart.register(...registerables);

  const COLORS = {
    accepted: '#16a34a',
    rejected: '#ef4444',
    waiting: '#f59e0b',
    blue: '#2563eb',
    purple: '#8b5cf6',
  };

  let loadingFilters = true;
  let loadingData = false;
  let error = '';

  let realms = [];
  let guilds = [];
  let difficulties = [];
  let raids = [];
  let bosses = [];

  let realm = '';
  let guild = '';
  let difficulty = '';
  let raid = '';
  let selectedBosses = [];

  let pulls = [];
  let lastUpdate = null;
  let autoRefresh = null;

  let acceptedWipesCanvas;
  let bestPullCanvas;
  let avgFightDpsCanvas;
  let wipesByBossCanvas;
  let statusDistributionCanvas;

  let acceptedWipesChart = null;
  let bestPullChart = null;
  let avgFightDpsChart = null;
  let wipesByBossChart = null;
  let statusDistributionChart = null;

  function normalizeFilterValue(value) {
    return value?.String ?? value ?? '';
  }

  function decodeSafe(value) {
    try {
      return decodeURIComponent(value);
    } catch {
      return value;
    }
  }

  function toNumber(value, fallback = 0) {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : fallback;
  }

  function getPullDps(pull) {
    const direct =
      pull.mean_dps ??
      pull.dps ??
      pull.raid_dps ??
      pull.damage_per_second ??
      pull.avg_dps;

    if (typeof direct === 'number' && Number.isFinite(direct)) {
      return direct;
    }

    const totalDamage = toNumber(pull.total_damage, 0);
    const durationSeconds = toNumber(pull.duration_ms, 0) / 1000;

    if (totalDamage > 0 && durationSeconds > 0) {
      return totalDamage / durationSeconds;
    }

    return 0;
  }

  async function loadFilters() {
    loadingFilters = true;
    error = '';

    try {
      const filterData = await fetchFilters();

      realms = (filterData.realms ?? []).map(normalizeFilterValue).filter(Boolean);
      guilds = (filterData.guilds ?? []).map(normalizeFilterValue).filter(Boolean);
      difficulties = (filterData.difficulties ?? []).map(normalizeFilterValue).filter(Boolean);
      raids = (filterData.raids ?? []).map(normalizeFilterValue).filter(Boolean);
      bosses = (filterData.bosses ?? []).map(normalizeFilterValue).filter(Boolean);

      if (!realm && realms[0]) realm = realms[0];
      if (!guild && guilds[0]) guild = guilds[0];
      if (!difficulty && difficulties[0]) difficulty = difficulties[0];
      if (!raid && raids[0]) raid = raids[0];
    } catch (err) {
      error = `Unable to load filters: ${err.message}`;
    } finally {
      loadingFilters = false;
    }
  }

  async function loadPulls() {
    if (!realm || !guild || !difficulty || !raid) return;

    loadingData = true;
    error = '';

    try {
      const data = await fetchPulls({
        realmSlug: realm,
        guildSlug: guild,
        difficulty,
        raidSlug: raid,
        bosses: selectedBosses.join(','),
        hideRejected: false,
      });

      pulls = Array.isArray(data) ? data : [];
      lastUpdate = new Date();
    } catch (err) {
      error = `Unable to load pulls: ${err.message}`;
      pulls = [];
    } finally {
      loadingData = false;
    }
  }

  function resetAutoRefresh() {
    if (autoRefresh) clearInterval(autoRefresh);
    if (realm && guild && difficulty && raid) {
      autoRefresh = setInterval(() => {
        loadPulls();
      }, 60000);
    }
  }

  function computeMetrics(data) {
    const ordered = [...data].sort((a, b) => toNumber(a.pull_count) - toNumber(b.pull_count));

    let acceptedCumulative = 0;
    let wipesCumulative = 0;
    let durationAccumulator = 0;
    let dpsAccumulator = 0;
    let bestPercent = Number.POSITIVE_INFINITY;

    const evolution = ordered.map((pull, index) => {
      const isAccepted = String(pull.status || '').toLowerCase() === 'accepted';
      const isWipe = pull.is_success === false;

      if (isAccepted) acceptedCumulative += 1;
      if (isWipe) wipesCumulative += 1;

      const overallPercent = toNumber(pull.overall_percent, 0);
      bestPercent = Math.min(bestPercent, overallPercent);

      const durationSeconds = toNumber(pull.duration_ms, 0) / 1000;
      durationAccumulator += durationSeconds;

      const dps = getPullDps(pull);
      dpsAccumulator += dps;

      return {
        attempt: toNumber(pull.pull_count, index + 1),
        accepted: acceptedCumulative,
        wipes: wipesCumulative,
        bestPercent: Number.isFinite(bestPercent) ? Number(bestPercent.toFixed(2)) : 0,
        avgFightTime: Number((durationAccumulator / (index + 1)).toFixed(2)),
        meanDps: Number((dpsAccumulator / (index + 1)).toFixed(2)),
      };
    });

    const byBoss = ordered.reduce((acc, pull) => {
      const boss = pull?.encounter?.boss?.slug || pull?.encounter?.slug || 'unknown';
      const bucket = acc.get(boss) || {
        boss,
        pulls: 0,
        accepted: 0,
        wipes: 0,
        bestPercent: Number.POSITIVE_INFINITY,
        totalDuration: 0,
        totalDps: 0,
      };

      bucket.pulls += 1;
      if (String(pull.status || '').toLowerCase() === 'accepted') bucket.accepted += 1;
      if (pull.is_success === false) bucket.wipes += 1;

      bucket.bestPercent = Math.min(bucket.bestPercent, toNumber(pull.overall_percent, 0));
      bucket.totalDuration += toNumber(pull.duration_ms, 0) / 1000;
      bucket.totalDps += getPullDps(pull);

      acc.set(boss, bucket);
      return acc;
    }, new Map());

    const bossRows = [...byBoss.values()].map((item) => ({
      boss: item.boss,
      pulls: item.pulls,
      accepted: item.accepted,
      wipes: item.wipes,
      bestPercent: Number.isFinite(item.bestPercent) ? Number(item.bestPercent.toFixed(2)) : 0,
      avgFightTime: Number((item.totalDuration / Math.max(item.pulls, 1)).toFixed(2)),
      meanDps: Number((item.totalDps / Math.max(item.pulls, 1)).toFixed(2)),
    }));

    const statusDistribution = [
      {
        name: 'Accepted',
        value: ordered.filter((pull) => String(pull.status || '').toLowerCase() === 'accepted').length,
      },
      {
        name: 'Rejected',
        value: ordered.filter((pull) => String(pull.status || '').toLowerCase() === 'rejected').length,
      },
      {
        name: 'Waiting',
        value: ordered.filter((pull) => String(pull.status || '').toLowerCase() === 'waiting').length,
      },
    ];

    const wipesByBoss = bossRows.map((row) => ({ boss: row.boss, wipes: row.wipes }));

    const totalDuration = ordered.reduce((sum, pull) => sum + toNumber(pull.duration_ms, 0) / 1000, 0);
    const totalDps = ordered.reduce((sum, pull) => sum + getPullDps(pull), 0);

    const bestPullRaw = ordered.length
      ? Math.min(...ordered.map((pull) => toNumber(pull.overall_percent, Number.POSITIVE_INFINITY)))
      : 0;

    return {
      evolution,
      statusDistribution,
      wipesByBoss,
      bossRows,
      kpis: {
        totalPulls: ordered.length,
        acceptedPulls: statusDistribution[0].value,
        totalWipes: ordered.filter((pull) => pull.is_success === false).length,
        bestPull: Number.isFinite(bestPullRaw) ? Number(bestPullRaw.toFixed(2)) : 0,
        avgFightTime: ordered.length ? Number((totalDuration / ordered.length).toFixed(2)) : 0,
        meanDps: ordered.length ? Number((totalDps / ordered.length).toFixed(2)) : 0,
      },
    };
  }

  function destroyCharts() {
    acceptedWipesChart?.destroy();
    bestPullChart?.destroy();
    avgFightDpsChart?.destroy();
    wipesByBossChart?.destroy();
    statusDistributionChart?.destroy();

    acceptedWipesChart = null;
    bestPullChart = null;
    avgFightDpsChart = null;
    wipesByBossChart = null;
    statusDistributionChart = null;
  }

  function renderCharts() {
    if (!acceptedWipesCanvas || !bestPullCanvas || !avgFightDpsCanvas || !wipesByBossCanvas || !statusDistributionCanvas) return;

    destroyCharts();

    const labels = metrics.evolution.map((point) => point.attempt);

    acceptedWipesChart = new Chart(acceptedWipesCanvas, {
      type: 'line',
      data: {
        labels,
        datasets: [
          {
            label: 'Accepted',
            data: metrics.evolution.map((point) => point.accepted),
            borderColor: COLORS.accepted,
            backgroundColor: `${COLORS.accepted}22`,
            borderWidth: 1.5,
            tension: 0.25,
            pointRadius: 2,
          },
          {
            label: 'Wipes',
            data: metrics.evolution.map((point) => point.wipes),
            borderColor: COLORS.rejected,
            backgroundColor: `${COLORS.rejected}22`,
            borderWidth: 1.5,
            tension: 0.25,
            pointRadius: 2,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          x: { title: { display: true, text: 'Attempt' } },
          y: { beginAtZero: true, title: { display: true, text: 'Count' } },
        },
      },
    });

    bestPullChart = new Chart(bestPullCanvas, {
      type: 'line',
      data: {
        labels,
        datasets: [
          {
            label: 'Best pull (%)',
            data: metrics.evolution.map((point) => point.bestPercent),
            borderColor: COLORS.blue,
            backgroundColor: `${COLORS.blue}33`,
            fill: true,
            borderWidth: 1.5,
            tension: 0.25,
            pointRadius: 2,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          x: { title: { display: true, text: 'Attempt' } },
          y: { beginAtZero: true, title: { display: true, text: '%' } },
        },
      },
    });

    avgFightDpsChart = new Chart(avgFightDpsCanvas, {
      type: 'line',
      data: {
        labels,
        datasets: [
          {
            label: 'Avg fight time (s)',
            data: metrics.evolution.map((point) => point.avgFightTime),
            borderColor: COLORS.waiting,
            borderWidth: 1.5,
            tension: 0.25,
            pointRadius: 2,
            yAxisID: 'y',
          },
          {
            label: 'Mean DPS',
            data: metrics.evolution.map((point) => point.meanDps),
            borderColor: COLORS.purple,
            borderWidth: 1.5,
            tension: 0.25,
            pointRadius: 2,
            yAxisID: 'y1',
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          x: { title: { display: true, text: 'Attempt' } },
          y: {
            beginAtZero: true,
            position: 'left',
            title: { display: true, text: 'Avg fight time (s)' },
          },
          y1: {
            beginAtZero: true,
            position: 'right',
            title: { display: true, text: 'Mean DPS' },
            grid: { drawOnChartArea: false },
          },
        },
      },
    });

    wipesByBossChart = new Chart(wipesByBossCanvas, {
      type: 'bar',
      data: {
        labels: metrics.wipesByBoss.map((row) => row.boss),
        datasets: [
          {
            label: 'Wipes',
            data: metrics.wipesByBoss.map((row) => row.wipes),
            backgroundColor: `${COLORS.rejected}aa`,
            borderColor: COLORS.rejected,
            borderWidth: 1,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          x: { ticks: { autoSkip: false, maxRotation: 45, minRotation: 0 } },
          y: { beginAtZero: true, title: { display: true, text: 'Wipes' } },
        },
      },
    });

    statusDistributionChart = new Chart(statusDistributionCanvas, {
      type: 'pie',
      data: {
        labels: metrics.statusDistribution.map((item) => item.name),
        datasets: [
          {
            data: metrics.statusDistribution.map((item) => item.value),
            backgroundColor: [COLORS.accepted, COLORS.rejected, COLORS.waiting],
            borderWidth: 1,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
      },
    });
  }

  $: metrics = computeMetrics(pulls);
  $: if (metrics) renderCharts();

  $: filterSignature = `${realm}|${guild}|${difficulty}|${raid}|${selectedBosses.join(',')}`;
  $: if (realm && guild && difficulty && raid && filterSignature) {
    resetAutoRefresh();
    loadPulls();
  }

  onMount(async () => {
    await loadFilters();
  });

  onDestroy(() => {
    if (autoRefresh) clearInterval(autoRefresh);
    destroyCharts();
  });
</script>

<main class="metrics-page">
  <div class="metrics-topbar">
    <div class="metrics-title">
      <h2>Pulls metrics</h2>
      <p>Last refresh: {lastUpdate ? lastUpdate.toLocaleTimeString() : 'never'}</p>
    </div>
  </div>

  <section class="card">
    <header class="card-header">
      <h3 class="card-title">Filters</h3>
      <p class="card-description">Select values available in the database.</p>
    </header>
    <div class="card-content">
      {#if loadingFilters}
        <p>Loading filter values…</p>
      {:else}
        <div class="filters-grid">
          <div class="field">
            <label for="realm-select">Realm</label>
            <select id="realm-select" class="select" bind:value={realm}>
              {#each realms as value}
                <option value={value}>{value}</option>
              {/each}
            </select>
          </div>

          <div class="field">
            <label for="guild-select">Guild</label>
            <select id="guild-select" class="select" bind:value={guild}>
              {#each guilds as value}
                <option value={value}>{decodeSafe(value)}</option>
              {/each}
            </select>
          </div>

          <div class="field">
            <label for="difficulty-select">Difficulty</label>
            <select id="difficulty-select" class="select" bind:value={difficulty}>
              {#each difficulties as value}
                <option value={value}>{value}</option>
              {/each}
            </select>
          </div>

          <div class="field">
            <label for="raid-select">Raid</label>
            <select id="raid-select" class="select" bind:value={raid}>
              {#each raids as value}
                <option value={value}>{value}</option>
              {/each}
            </select>
          </div>

          <div class="field">
            <label for="boss-select">Bosses (multi-select)</label>
            <select id="boss-select" class="select" multiple bind:value={selectedBosses}>
              {#each bosses as value}
                <option value={value}>{value}</option>
              {/each}
            </select>
            <div class="helper">Use Ctrl/Cmd + click for multiple bosses.</div>
          </div>

          <div class="filter-actions">
            <button type="button" class="button primary" on:click={loadPulls} disabled={loadingData}>
              {loadingData ? 'Refreshing…' : 'Refresh now'}
            </button>
            <button type="button" class="button ghost" on:click={() => (selectedBosses = [])}>
              Reset bosses
            </button>
          </div>
        </div>
      {/if}
    </div>
  </section>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <section class="card">
    <header class="card-header">
      <h3 class="card-title">Key indicators</h3>
      <p class="card-description">Accepted pulls, wipes, best pull, average fight time and mean DPS.</p>
    </header>
    <div class="card-content">
      <div class="kpi-grid">
        <div class="kpi"><div class="kpi-label">Total pulls</div><div class="kpi-value">{metrics.kpis.totalPulls}</div></div>
        <div class="kpi"><div class="kpi-label">Accepted pulls</div><div class="kpi-value">{metrics.kpis.acceptedPulls}</div></div>
        <div class="kpi"><div class="kpi-label">Wipes</div><div class="kpi-value">{metrics.kpis.totalWipes}</div></div>
        <div class="kpi"><div class="kpi-label">Best pull (%)</div><div class="kpi-value">{metrics.kpis.bestPull}</div></div>
        <div class="kpi"><div class="kpi-label">Avg fight time (s)</div><div class="kpi-value">{metrics.kpis.avgFightTime}</div></div>
        <div class="kpi"><div class="kpi-label">Mean DPS</div><div class="kpi-value">{metrics.kpis.meanDps}</div></div>
      </div>
    </div>
  </section>

  <div class="chart-grid">
    <section class="card">
      <header class="card-header">
        <h3 class="card-title">Evolution of accepted pulls & wipes</h3>
      </header>
      <div class="card-content">
        <div class="chart-box">
          {#if metrics.evolution.length}
            <canvas bind:this={acceptedWipesCanvas}></canvas>
          {:else}
            <p class="chart-empty">No data</p>
          {/if}
        </div>
      </div>
    </section>

    <section class="card">
      <header class="card-header">
        <h3 class="card-title">Best pull evolution</h3>
      </header>
      <div class="card-content">
        <div class="chart-box">
          {#if metrics.evolution.length}
            <canvas bind:this={bestPullCanvas}></canvas>
          {:else}
            <p class="chart-empty">No data</p>
          {/if}
        </div>
      </div>
    </section>

    <section class="card">
      <header class="card-header">
        <h3 class="card-title">Average fight time and mean DPS</h3>
      </header>
      <div class="card-content">
        <div class="chart-box">
          {#if metrics.evolution.length}
            <canvas bind:this={avgFightDpsCanvas}></canvas>
          {:else}
            <p class="chart-empty">No data</p>
          {/if}
        </div>
      </div>
    </section>

    <section class="card">
      <header class="card-header">
        <h3 class="card-title">Wipes by boss</h3>
      </header>
      <div class="card-content">
        <div class="chart-box">
          {#if metrics.wipesByBoss.length}
            <canvas bind:this={wipesByBossCanvas}></canvas>
          {:else}
            <p class="chart-empty">No data</p>
          {/if}
        </div>
      </div>
    </section>

    <section class="card">
      <header class="card-header">
        <h3 class="card-title">Pull status distribution</h3>
      </header>
      <div class="card-content">
        <div class="chart-box">
          {#if metrics.statusDistribution.some((item) => item.value > 0)}
            <canvas bind:this={statusDistributionCanvas}></canvas>
          {:else}
            <p class="chart-empty">No data</p>
          {/if}
        </div>
      </div>
    </section>
  </div>

  <section class="card">
    <header class="card-header">
      <h3 class="card-title">Boss summary table</h3>
      <p class="card-description">Pull count, accepted pulls, wipes, best pull %, average fight duration and mean DPS.</p>
    </header>
    <div class="card-content table-wrap">
      <table class="table">
        <thead>
          <tr>
            <th>Boss</th>
            <th>Pulls</th>
            <th>Accepted</th>
            <th>Wipes</th>
            <th>Best pull (%)</th>
            <th>Avg fight time (s)</th>
            <th>Mean DPS</th>
          </tr>
        </thead>
        <tbody>
          {#each metrics.bossRows as row}
            <tr>
              <td>{row.boss}</td>
              <td>{row.pulls}</td>
              <td>{row.accepted}</td>
              <td>{row.wipes}</td>
              <td>{row.bestPercent}</td>
              <td>{row.avgFightTime}</td>
              <td>{row.meanDps}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </section>
</main>

<style>
  .metrics-page {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .metrics-topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .metrics-title h2 {
    margin: 0;
    font-size: 1.5rem;
    color: #0f172a;
  }

  .metrics-title p {
    margin: 0.25rem 0 0;
    color: #475569;
    font-size: 0.95rem;
  }

  .card {
    border: 1px solid #e2e8f0;
    border-radius: 0.75rem;
    background: #fff;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.04);
  }

  .card-header { padding: 1rem 1rem 0.25rem; }
  .card-title { margin: 0; font-size: 1rem; color: #0f172a; }
  .card-description { margin: 0.25rem 0 0; color: #64748b; font-size: 0.85rem; }
  .card-content { padding: 1rem; }

  .filters-grid {
    display: grid;
    grid-template-columns: repeat(5, minmax(150px, 1fr));
    gap: 0.75rem;
  }

  .field { display: flex; flex-direction: column; gap: 0.4rem; }
  .field label { font-size: 0.8rem; color: #475569; font-weight: 600; }

  .select, .button {
    border: 1px solid #cbd5e1;
    border-radius: 0.5rem;
    background: #fff;
    color: #0f172a;
    font: inherit;
  }

  .select { padding: 0.55rem 0.7rem; }
  .select[multiple] { min-height: 130px; }

  .button { padding: 0.6rem 0.8rem; cursor: pointer; font-weight: 600; }
  .button.primary { background: #0f172a; color: #fff; border-color: #0f172a; }
  .button.ghost:hover { background: #f1f5f9; }
  .button:disabled { opacity: 0.55; cursor: not-allowed; }

  .filter-actions { display: flex; align-items: flex-end; gap: 0.5rem; }

  .kpi-grid {
    display: grid;
    grid-template-columns: repeat(6, minmax(110px, 1fr));
    gap: 0.75rem;
  }

  .kpi { border: 1px solid #e2e8f0; border-radius: 0.6rem; padding: 0.75rem; background: #f8fafc; }
  .kpi-label { font-size: 0.8rem; color: #64748b; }
  .kpi-value { margin-top: 0.35rem; font-size: 1.15rem; color: #0f172a; font-weight: 700; }

  .chart-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1rem;
  }

  .chart-box {
    height: 240px;
    border: 1px dashed #cbd5e1;
    border-radius: 0.5rem;
    padding: 0.5rem;
  }

  .chart-box canvas { width: 100% !important; height: 100% !important; }

  .chart-empty { color: #64748b; font-size: 0.9rem; }

  .table-wrap { overflow-x: auto; }
  .table { width: 100%; border-collapse: collapse; }
  .table th, .table td { border-bottom: 1px solid #e2e8f0; text-align: left; padding: 0.7rem; font-size: 0.9rem; }
  .table th { color: #334155; background: #f8fafc; font-weight: 600; }

  .helper { color: #64748b; font-size: 0.8rem; margin-top: 0.3rem; }
  .error { color: #b91c1c; background: #fee2e2; border: 1px solid #fecaca; border-radius: 0.5rem; padding: 0.7rem; }

  @media (max-width: 1200px) {
    .filters-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
    .kpi-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  }

  @media (max-width: 860px) {
    .filters-grid, .chart-grid, .kpi-grid { grid-template-columns: 1fr; }
  }
</style>
