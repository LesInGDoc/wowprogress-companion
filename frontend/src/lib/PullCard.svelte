<script>
  import { updatePull } from '../services/api.js';

  export let pull;
  export let authToken = '';
  export let onUpdate = () => {};
  export let ordinalNumber = 0;

  let isUpdating = false;
  let error = null;

  $: status = pull.status || 'pending';

  function getStatusColor(status) {
    switch (status?.toLowerCase()) {
      case 'accepted':
        return '#22c55e';
      case 'rejected':
        return '#ef4444';
      default:
        return '#f59e0b';
    }
  }

  async function handleUpdate(action) {
    if (!authToken.trim()) {
      error = 'Auth token required';
      return;
    }

    isUpdating = true;
    error = null;

    try {
      await updatePull(pull.pull_id, action, authToken);
      onUpdate();
    } catch (err) {
      error = err.message;
    } finally {
      isUpdating = false;
    }
  }
</script>

<div class="pull-card">
  <div class="pull-header">
    <div class="pull-info">
      <h3>Pull #{ordinalNumber}</h3>
      <p class="pull-count">Attempt: {pull.pull_count || 0}</p>
      {#if pull.overall_percent !== undefined && pull.overall_percent !== null}
        <p class="overall-percent">Health: <strong>{pull.overall_percent.toFixed(2)}%</strong></p>
      {/if}
      {#if pull.duration_ms}
        <p class="duration">Duration: {(pull.duration_ms / 1000).toFixed(1)}s</p>
      {/if}
      {#if pull.is_success !== undefined}
        <p class="kill-status" style="color: {pull.is_success ? '#22c55e' : '#666'}">
          {pull.is_success ? '✓ Kill' : 'Wipe'}
        </p>
      {/if}
    </div>
    
    <div class="status-badge" style="background-color: {getStatusColor(status)}">
      {status}
    </div>
  </div>

  {#if authToken}
    <div class="actions">
      <button 
        class="btn btn-accept" 
        on:click={() => handleUpdate('accept')}
        disabled={isUpdating}
      >
        ✓ Accept
      </button>
      <button 
        class="btn btn-reject" 
        on:click={() => handleUpdate('reject')}
        disabled={isUpdating}
      >
        ✗ Reject
      </button>
    </div>
  {/if}

  {#if error}
    <p class="error">{error}</p>
  {/if}
</div>

<style>
  .pull-card {
    background: white;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
    margin-bottom: 1rem;
  }

  @media (max-width: 768px) {
    .pull-card {
      padding: 1rem;
    }
  }

  .pull-header {
    display: flex;
    justify-content: space-between;
    align-items: start;
    margin-bottom: 1rem;    flex-wrap: wrap;
    gap: 1rem;
  }

  @media (max-width: 768px) {
    .pull-header {
      flex-direction: column;
      align-items: stretch;
    }  }

  .pull-info h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
  }

  .pull-info p {
    margin: 0.25rem 0;
    color: #666;
    font-size: 0.9rem;
  }

  .overall-percent {
    font-size: 1rem !important;
    color: #3b82f6 !important;
  }

  .overall-percent strong {
    font-size: 1.1rem;
  }

  .pull-count {
    font-weight: 500;
    color: #888 !important;
    font-size: 0.85rem !important;
  }

  .kill-status {
    font-weight: 600;
  }

  .boss-name {
    font-weight: 600;
  }

  .status-badge {
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    color: white;
    font-size: 0.85rem;
    font-weight: 600;
    text-transform: capitalize;
  }

  .actions {
    display: flex;
    gap: 0.75rem;
    margin-top: 1rem;
  }

  @media (max-width: 768px) {
    .actions {
      flex-direction: column;
    }
  }

  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  @media (max-width: 768px) {
    .btn {
      width: 100%;
    }
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-accept {
    background: #22c55e;
    color: white;
  }

  .btn-accept:hover:not(:disabled) {
    background: #16a34a;
  }

  .btn-reject {
    background: #ef4444;
    color: white;
  }

  .btn-reject:hover:not(:disabled) {
    background: #dc2626;
  }

  .error {
    color: #ef4444;
    font-size: 0.85rem;
    margin-top: 0.5rem;
  }
</style>
