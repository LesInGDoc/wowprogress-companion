<script>
  import { onMount, onDestroy } from 'svelte';
  import { fetchPulls, fetchFilters } from './services/api.js';
  import PullCard from './lib/PullCard.svelte';

  let realmSlug = '';
  let guildSlug = '';
  let difficulty = '';
  let raidSlug = '';
  let bosses = [];
  let authToken = '';
  let hideRejected = true;

  // Available filter options
  let availableRealms = [];
  let availableGuilds = [];
  let availableRaids = [];
  let availableBosses = [];
  let availableDifficulties = [];

  let pulls = [];
  let isLoading = false;
  let isLoadingFilters = true;
  let error = null;

  // Auto-update functionality
  let lastUpdateTime = null;
  let autoUpdateInterval = null;
  let reverseOrder = false;

  onMount(async () => {
    try {
      const filters = await fetchFilters();
      availableRealms = filters.realms.map(r => r.String || r);
      availableGuilds = filters.guilds.map(g => g.String || g);
      availableRaids = filters.raids.map(r => r.String || r);
      availableBosses = filters.bosses.map(b => b.String || b);
      availableDifficulties = filters.difficulties.map(d => d.String || d);
      
      // Set default values if available
      if (availableRealms.length > 0) realmSlug = availableRealms[0];
      if (availableGuilds.length > 0) guildSlug = availableGuilds[0];
      if (availableDifficulties.length > 0) difficulty = availableDifficulties[0];
      if (availableRaids.length > 0) raidSlug = availableRaids[0];
    } catch (err) {
      error = 'Failed to load filters: ' + err.message;
    } finally {
      isLoadingFilters = false;
    }
  });

  onDestroy(() => {
    if (autoUpdateInterval) {
      clearInterval(autoUpdateInterval);
    }
  });

  async function handleFetchPulls() {
    isLoading = true;
    error = null;

    try {
      pulls = await fetchPulls({
        realmSlug,
        guildSlug,
        difficulty,
        raidSlug,
        bosses: bosses.join(','),
        hideRejected
      });
      lastUpdateTime = new Date();
      startAutoUpdate();
    } catch (err) {
      error = err.message;
      pulls = [];
    } finally {
      isLoading = false;
    }
  }

  function startAutoUpdate() {
    // Clear existing interval
    if (autoUpdateInterval) {
      clearInterval(autoUpdateInterval);
    }
    // Start new interval for 30 seconds
    autoUpdateInterval = setInterval(() => {
      if (realmSlug && guildSlug && raidSlug && !isLoading) {
        handleFetchPulls();
      }
    }, 30000);
  }

  function toggleReverseOrder() {
    reverseOrder = !reverseOrder;
  }

  function formatLastUpdate(date) {
    if (!date) return 'Never';
    return date.toLocaleTimeString();
  }

  $: displayedPulls = reverseOrder ? [...pulls].reverse() : pulls;
</script>

<main>
  <h1>WoW Progress Companion</h1>

  <div class="container">
    <section class="form-section">
      <h2>Query Parameters</h2>
      
      {#if isLoadingFilters}
        <p>Loading available options...</p>
      {:else}
        <div class="form-grid">
          <div class="form-group">
            <label for="realm">Realm</label>
            <select id="realm" bind:value={realmSlug}>
              {#each availableRealms as realm}
                <option value={realm}>{realm}</option>
              {/each}
            </select>
          </div>

          <div class="form-group">
            <label for="guild">Guild</label>
            <select id="guild" bind:value={guildSlug}>
              {#each availableGuilds as guild}
                <option value={guild}>{decodeURIComponent(guild)}</option>
              {/each}
            </select>
          </div>

          <div class="form-group">
            <label for="difficulty">Difficulty</label>
            <select id="difficulty" bind:value={difficulty}>
              {#each availableDifficulties as diff}
                <option value={diff}>{diff.charAt(0).toUpperCase() + diff.slice(1)}</option>
              {/each}
            </select>
          </div>

          <div class="form-group">
            <label for="raid">Raid</label>
            <select id="raid" bind:value={raidSlug}>
              {#each availableRaids as raid}
                <option value={raid}>{raid}</option>
              {/each}
            </select>
          </div>

          <div class="form-group full-width">
            <label for="bosses">Bosses (optional, select multiple)</label>
            <select id="bosses" bind:value={bosses} multiple size="4">
              {#each availableBosses as boss}
                <option value={boss}>{boss}</option>
              {/each}
            </select>
            <small class="hint">Hold Ctrl/Cmd to select multiple bosses</small>
          </div>

          <div class="form-group full-width">
            <label for="token">Auth Token (for updates)</label>
            <input 
              id="token"
              type="password" 
              bind:value={authToken} 
              placeholder="Enter your authentication token"
            />
          </div>

          <div class="form-group full-width">
            <label class="checkbox-label">
              <input 
                type="checkbox" 
                bind:checked={hideRejected}
              />
              Hide rejected pulls
            </label>
          </div>
        </div>
      {/if}

      <button 
        class="fetch-btn" 
        on:click={handleFetchPulls}
        disabled={isLoading || !realmSlug || !guildSlug || !raidSlug}
      >
        {isLoading ? 'Loading...' : 'Fetch Pulls'}
      </button>

      {#if error}
        <p class="error-message">{error}</p>
      {/if}
    </section>

    <section class="pulls-section">
      <div class="pulls-header">
        <h2>Pulls ({pulls.length})</h2>
        <div class="pulls-controls">
          {#if lastUpdateTime}
            <span class="last-update">Last update: {formatLastUpdate(lastUpdateTime)}</span>
          {/if}
          <button 
            class="control-btn update-btn" 
            on:click={handleFetchPulls}
            disabled={isLoading || !realmSlug || !guildSlug || !raidSlug}
            title="Refresh data"
          >
            🔄 Update
          </button>
          <button 
            class="control-btn reverse-btn" 
            on:click={toggleReverseOrder}
            disabled={pulls.length === 0}
            title="Reverse order"
          >
            ⇅ {reverseOrder ? 'Oldest First' : 'Newest First'}
          </button>
        </div>
      </div>
      
      {#if pulls.length === 0 && !isLoading}
        <div class="empty-state">
          <p>No pulls found</p>
          <p class="empty-hint">Configure parameters above and click "Fetch Pulls"</p>
        </div>
      {:else}
        <div class="pulls-list">
          {#each displayedPulls as pull, index (pull.pull_id)}
            <PullCard 
              {pull} 
              {authToken}
              ordinalNumber={reverseOrder ? index + 1 : pulls.length - index}
              onUpdate={handleFetchPulls}
            />
          {/each}
        </div>
      {/if}
    </section>
  </div>
</main>

<style>
  main {
    min-height: 100vh;
  }

  h1 {
    font-size: 2rem;
    margin-bottom: 2rem;
    color: #1a1a1a;
  }

  @media (max-width: 768px) {
    h1 {
      font-size: 1.5rem;
      margin-bottom: 1rem;
    }
  }

  h2 {
    font-size: 1.5rem;
    margin-bottom: 1.5rem;
    color: #333;
  }

  @media (max-width: 768px) {
    h2 {
      font-size: 1.25rem;
      margin-bottom: 1rem;
    }
  }

  .container {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: 2rem;
  }

  @media (max-width: 1024px) {
    .container {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 768px) {
    .container {
      gap: 1rem;
    }
  }

  .form-section {
    background: white;
    padding: 2rem;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    height: fit-content;
    position: sticky;
    top: 2rem;
  }

  @media (max-width: 768px) {
    .form-section {
      position: static;
      padding: 1rem;
    }
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  @media (max-width: 768px) {
    .form-grid {
      grid-template-columns: 1fr;
      gap: 0.75rem;
    }
  }

  .form-group {
    display: flex;
    flex-direction: column;
  }

  .form-group.full-width {
    grid-column: 1 / -1;
  }

  label {
    font-weight: 600;
    margin-bottom: 0.5rem;
    color: #333;
    font-size: 0.9rem;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 500;
    cursor: pointer;
    margin-bottom: 0;
  }

  .checkbox-label input[type="checkbox"] {
    width: auto;
    cursor: pointer;
  }

  input, select {
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
  }

  select[multiple] {
    padding: 0.5rem;
  }

  .hint {
    display: block;
    margin-top: 0.25rem;
    font-size: 0.8rem;
    color: #666;
  }

  input:focus, select:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .fetch-btn {
    width: 100%;
    padding: 1rem;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }

  .fetch-btn:hover:not(:disabled) {
    background: #2563eb;
  }

  .fetch-btn:disabled {
    background: #93c5fd;
    cursor: not-allowed;
  }

  .error-message {
    color: #ef4444;
    margin-top: 1rem;
    padding: 0.75rem;
    background: #fee2e2;
    border-radius: 6px;
    font-size: 0.9rem;
  }

  .pulls-section {
    min-height: 400px;
  }

  .pulls-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .pulls-controls {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .last-update {
    font-size: 0.9rem;
    color: #666;
    padding: 0.5rem 0.75rem;
    background: #f3f4f6;
    border-radius: 6px;
  }

  .control-btn {
    padding: 0.5rem 1rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    background: white;
    color: #333;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .control-btn:hover:not(:disabled) {
    background: #f9fafb;
    border-color: #3b82f6;
    color: #3b82f6;
  }

  .control-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .update-btn:hover:not(:disabled) {
    background: #3b82f6;
    color: white;
  }

  @media (max-width: 768px) {
    .pulls-header {
      flex-direction: column;
      align-items: flex-start;
    }

    .pulls-controls {
      width: 100%;
      justify-content: space-between;
    }

    .last-update {
      width: 100%;
      text-align: center;
    }

    .control-btn {
      flex: 1;
      min-width: calc(50% - 0.375rem);
    }
  }

  .pulls-list {
    display: flex;
    flex-direction: column;
  }

  .empty-state {
    text-align: center;
    padding: 3rem;
    color: #999;
  }

  .empty-state p:first-child {
    font-size: 1.25rem;
    margin-bottom: 0.5rem;
  }

  .empty-hint {
    font-size: 0.9rem;
  }
</style>
