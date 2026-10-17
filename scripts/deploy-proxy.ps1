# Deploy the JARVIS proxy (Cloudflare Worker) that holds the Groq key.
#
# Usage:
#   ./scripts/deploy-proxy.ps1 -GroqKey "gsk_..." [-ProxyToken "shared-secret"]
#
# Requires a Cloudflare account. The first run opens a browser to log in and may
# ask you to pick/create a Worker name. Prints the deployed URL at the end.
#
#   -GroqKey     Groq API key (https://console.groq.com/keys). Required.
#   -ProxyToken  Optional shared secret. If set, put this same value in the
#                app's "API key" field; requests without it get HTTP 401.
param(
    [Parameter(Mandatory = $true)][string]$GroqKey,
    [string]$ProxyToken
)

$ErrorActionPreference = 'Stop'
$proxy = Join-Path $PSScriptRoot '..\proxy'

Push-Location $proxy
try {
    if (-not (Test-Path 'node_modules')) {
        Write-Host 'Installing proxy dependencies...'
        npm install
        if (-not $?) { throw 'npm install failed' }
    }

    Write-Host 'Storing GROQ_API_KEY secret...'
    $GroqKey | npx wrangler secret put GROQ_API_KEY
    if (-not $?) { throw 'Failed to store GROQ_API_KEY' }

    if ($ProxyToken) {
        Write-Host 'Storing PROXY_TOKEN secret...'
        $ProxyToken | npx wrangler secret put PROXY_TOKEN
        if (-not $?) { throw 'Failed to store PROXY_TOKEN' }
    }

    Write-Host 'Deploying worker...'
    npx wrangler deploy
    if (-not $?) { throw 'wrangler deploy failed' }

    Write-Host ''
    Write-Host 'Done. Copy the "https://...workers.dev" URL printed above into the'
    Write-Host 'app: gear icon -> Base URL, model "openai/gpt-oss-20b", and the'
    Write-Host 'PROXY_TOKEN (if you set one) as the API key.'
}
finally {
    Pop-Location
}
