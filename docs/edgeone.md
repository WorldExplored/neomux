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
EDGEONE_GATEWAY_BASE_URL=https://ai-gateway.edgeone.link/v1
```

When CLI `EDGEONE_BASE_URL` is a deployed EdgeOne domain, neomux sends chat requests to `/api/chat`. When it is the direct Makers Models gateway, neomux sends requests to `/v1/chat/completions`.

Hosted routes:

- `/api/chat`: EdgeOne model proxy
- `/api/models`: static model metadata
- `/api/usage`: placeholder usage/pricing status
- `/`: install and demo page
