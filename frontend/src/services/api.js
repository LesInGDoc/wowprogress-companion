const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export class APIError extends Error {
  constructor(message, status) {
    super(message);
    this.status = status;
    this.name = 'APIError';
  }
}

export async function fetchPulls({ realmSlug, guildSlug, difficulty, raidSlug, bosses, hideRejected }) {
  const params = new URLSearchParams({
    realm_slug: realmSlug,
    guild_slug: guildSlug,
    difficulty: difficulty,
    raid_slug: raidSlug
  });

  if (bosses && bosses.trim()) {
    params.append('bosses', bosses);
  }

  if (hideRejected) {
    params.append('hide_rejected', 'true');
  }

  const response = await fetch(`${API_BASE_URL}/pulls?${params}`);

  if (!response.ok) {
    throw new APIError(`Failed to fetch pulls: ${response.statusText}`, response.status);
  }

  return await response.json();
}

export async function updatePull(pullId, action, authToken) {
  const response = await fetch(`${API_BASE_URL}/pulls/${pullId}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${authToken}`
    },
    body: JSON.stringify({ action })
  });

  if (response.status === 401) {
    throw new APIError('Unauthorized: Invalid token', 401);
  }

  if (response.status === 404) {
    throw new APIError('Pull not found', 404);
  }

  if (!response.ok) {
    throw new APIError(`Failed to update pull: ${response.statusText}`, response.status);
  }
}

export async function fetchFilters() {
  const response = await fetch(`${API_BASE_URL}/pulls/filters`);

  if (!response.ok) {
    throw new APIError(`Failed to fetch filters: ${response.statusText}`, response.status);
  }

  return await response.json();
}
