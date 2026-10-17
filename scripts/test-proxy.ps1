# Smoke-test the deployed JARVIS proxy: POST to /chat/completions with a tiny
# request to verify model pinning, streaming support and (optionally) auth.
#
# Usage:
#   ./scripts/test-proxy.ps1 -Url "https://habit-planner-ai-proxy.workers.dev" [-Token "shared-secret"]
param(
    [Parameter(Mandatory = $true)][string]$Url,
    [string]$Token
)

$ErrorActionPreference = 'Stop'
$endpoint = ($Url.TrimEnd('/') + '/chat/completions')

$headers = @{ 'content-type' = 'application/json' }
if ($Token) {
    $headers['authorization'] = "Bearer $Token"
}

$body = @{
    model    = 'should-be-pinned'
    messages = @(
        @{ role = 'user'; content = 'Say "ok"' }
    )
    max_tokens = 8
    stream     = $true
} | ConvertTo-Json -Depth 3

Write-Host "POST $endpoint ..."
try {
    $response = Invoke-WebRequest -Uri $endpoint -Method POST -Headers $headers -Body $body -TimeoutSec 60 -UseBasicParsing -MaximumRedirection 0
    Write-Host "Status: $($response.StatusCode)"
    $first = $response.Content.Substring(0, [Math]::Min(200, $response.Content.Length))
    Write-Host "First chunk of body (SSE):"
    Write-Host $first
}
catch {
    if ($_.Exception.Response) {
        $web = $_.Exception.Response
        $reader = New-Object System.IO.StreamReader($web.GetResponseStream())
        $text = $reader.ReadToEnd()
        Write-Error ("Status: $($web.StatusCode) - $($web.StatusDescription)`n$text")
    } else {
        Write-Error $_
    }
}
