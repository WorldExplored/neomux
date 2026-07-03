# EdgeOne Deployment

Use these settings in Tencent EdgeOne Makers:

```json
{
  "installCommand": "npm install",
  "buildCommand": "npm run edgeone:build",
  "outputDirectory": "dist",
  "nodeVersion": "22.11.0"
}
```

Build locally:

```bash
npm run edgeone:build
```

Deploy from CI or a logged-in machine:

```bash
edgeone makers deploy ./dist -n neomux -t $EDGEONE_API_TOKEN -e production
```

Required environment variable:

```bash
MAKERS_MODELS_KEY
```

Optional:

```bash
EDGEONE_BASE_URL=https://ai-gateway.edgeone.link/v1
```

The local CLI sends chat requests to `EDGEONE_BASE_URL/chat/completions`. The hosted `/api/chat` route is the EdgeOne proxy endpoint for server/demo use.

Hosted routes:

- `/api/chat`: EdgeOne model proxy
- `/api/models`: static model metadata
- `/api/usage`: placeholder usage/pricing status
- `/`: install and demo page
