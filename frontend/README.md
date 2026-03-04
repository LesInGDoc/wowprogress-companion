# WoW Progress Companion - Svelte Frontend

A modern Svelte application for managing WoW raid pulls via the WoW Progress Companion API.

## Features

- **Fetch Pulls**: Query pulls with configurable parameters:
  - Realm slug
  - Guild slug (automatically URL-encoded)
  - Difficulty (Normal, Heroic, Mythic)
  - Raid slug
  - Boss filters (optional, comma-separated)

- **Update Pull Status**: Accept or reject pulls with authentication
  - Secure token field for API authentication
  - Visual status indicators (green for accepted, red for rejected)
  - Real-time updates

- **Modern UI**: Clean, responsive interface built with Svelte

## Project Structure

```
frontend/
├── src/
│   ├── App.svelte              # Main application component
│   ├── main.js                 # Application entry point
│   ├── app.css                 # Global styles
│   ├── lib/
│   │   └── PullCard.svelte     # Pull card component
│   └── services/
│       └── api.js              # API client
├── index.html
├── package.json
├── vite.config.js
├── svelte.config.js
├── Dockerfile
└── nginx.conf
```

## Requirements

- Node.js 20+
- npm or yarn

## Development

1. Install dependencies:
```bash
npm install
```

2. Create a `.env` file (optional):
```bash
cp .env.example .env
```

3. Start the development server:
```bash
npm run dev
```

The app will be available at `http://localhost:5173`

## Build

Build for production:
```bash
npm run build
```

Preview production build:
```bash
npm run preview
```

## Docker

Build and run with Docker:
```bash
docker build -t wowprogress-frontend .
docker run -p 5173:80 wowprogress-frontend
```

Or use docker-compose:
```bash
docker compose up frontend
```

The frontend will be available at `http://localhost:5173`

## Environment Variables

- `VITE_API_URL`: API base URL (default: `http://localhost:3000`)

## Usage

1. **Configure Connection**: The app connects to the API at the configured URL
2. **Set Query Parameters**: Fill in realm slug, guild slug, difficulty, and raid slug
3. **Optional Filters**: Add comma-separated boss slugs to filter specific bosses
4. **Fetch Pulls**: Click the "Fetch Pulls" button to retrieve data
5. **Update Status**: Enter your auth token and use Accept/Reject buttons

## Example Configuration

```
Realm Slug: ysondre
Guild Slug: WoW Hôtel
Difficulty: mythic
Raid Slug: nerubar-palace
Bosses: the-silken-court,rashanan
Auth Token: your-secret-token
```

Note: The guild slug is automatically URL-encoded when sent to the API.
