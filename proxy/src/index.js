// Tiny Cloudflare Worker that fronts an OpenAI-compatible provider (Groq) with
// a server-side API key. The app points its assistant "Base URL" here, so the
// key never ships inside the desktop/phone build and never reaches the client.
//
// It forwards `POST .../chat/completions` verbatim (streaming included) and only
// rewrites the `model` so a shared free key cannot be pointed at a costly model.

const UPSTREAM = 'https://api.groq.com/openai/v1/chat/completions'
const DEFAULT_MODEL = 'openai/gpt-oss-20b'

/** Headers copied straight back from the upstream provider. */
const PASSTHROUGH = ['content-type', 'retry-after']

function cors() {
  return {
    'access-control-allow-origin': '*',
    'access-control-allow-methods': 'POST, OPTIONS',
    'access-control-allow-headers': 'content-type, authorization',
  }
}

function error(message, status) {
  return new Response(JSON.stringify({ error: { message } }), {
    status,
    headers: { 'content-type': 'application/json', ...cors() },
  })
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url)

    if (request.method === 'OPTIONS') {
      return new Response(null, { status: 204, headers: cors() })
    }
    if (request.method === 'GET' && url.pathname === '/') {
      return new Response(JSON.stringify({ ok: true, service: 'habit-planner-ai-proxy' }), {
        status: 200,
        headers: { 'content-type': 'application/json', ...cors() },
      })
    }
    if (request.method !== 'POST' || !url.pathname.endsWith('/chat/completions')) {
      return error('Not found', 404)
    }
    if (!env.GROQ_API_KEY) {
      return error('Proxy is missing its GROQ_API_KEY secret', 500)
    }
    // Optional shared token: when set, clients must send it as their API key.
    if (env.PROXY_TOKEN) {
      const auth = request.headers.get('authorization') ?? ''
      if (auth !== `Bearer ${env.PROXY_TOKEN}`) {
        return error('Unauthorized', 401)
      }
    }

    let payload
    try {
      payload = await request.json()
    } catch {
      return error('Request body must be JSON', 400)
    }

    // Pin the model so a shared key cannot be abused for other/expensive models.
    payload.model = env.MODEL || DEFAULT_MODEL

    const upstream = await fetch(UPSTREAM, {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        authorization: `Bearer ${env.GROQ_API_KEY}`,
      },
      body: JSON.stringify(payload),
    })

    // Stream the body straight through (SSE included); keep the status so the
    // client's own retry logic still sees 429 / 5xx and Retry-After.
    const headers = new Headers(cors())
    for (const name of PASSTHROUGH) {
      const value = upstream.headers.get(name)
      if (value) headers.set(name, value)
    }

    return new Response(upstream.body, { status: upstream.status, headers })
  },
}
