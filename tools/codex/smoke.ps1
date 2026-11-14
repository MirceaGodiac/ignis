<#
Send one message through a running IGNIS server and follow the episode until the turn ends (Windows).

  tools\codex\smoke.ps1                                   # http://127.0.0.1:8787/api/v1
  tools\codex\smoke.ps1 -Base http://127.0.0.1:8788/api/v1 -Text 'tool approval' -Approve
  tools\codex\smoke.ps1 -SpendTokens                      # required when the runtime is the real Codex

Approvals the runtime asks for are resolved with -Approve (default: deny). Exit code 0 when the
episode reached `completed`.
#>
param(
    [string]$Base = 'http://127.0.0.1:8787/api/v1',
    [string]$Text = 'Reply with one short sentence: what is IGNIS?',
    [switch]$Approve,
    [switch]$SpendTokens,
    [int]$TimeoutSeconds = 180
)
$ErrorActionPreference = 'Stop'
$Decision = if ($Approve) { 'approved' } else { 'denied' }

function Post($Path, $Body) {
    Invoke-RestMethod -Method Post -Uri "$Base$Path" -ContentType 'application/json' -Body ($Body | ConvertTo-Json -Compress)
}

$caps = Invoke-RestMethod "$Base/capabilities"
if (-not $caps.agent_runtime.available) { [Console]::Error.WriteLine("agent runtime unavailable: $($caps.agent_runtime.detail)"); exit 1 }
$ua = $caps.runtime_build.user_agent
Write-Host "runtime: $($caps.runtime_build.runtime_build_id)  ($ua)"
if (-not $ua.StartsWith('ignis-fake-codex') -and -not $SpendTokens) {
    [Console]::Error.WriteLine('this server runs the REAL Codex: a message calls the model and spends tokens. Re-run with -SpendTokens.')
    exit 3
}

$run = "smoke-$([DateTimeOffset]::UtcNow.ToUnixTimeSeconds())-$PID"
$ep = (Post '/episodes' @{ goal = "Smoke test $run"; client_request_id = "$run-ep" }).episode_id
Write-Host "episode: $ep"
Post "/episodes/$ep/messages" @{ text = $Text; client_request_id = "$run-msg" } | Out-Null
Write-Host "sent:    $Text"

$cursor = 0
$status = ''
$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
while ((Get-Date) -lt $deadline) {
    # Read what the log holds after the cursor; the stream stays open, so cut it after 2 s
    # (curl.exe ships with Windows 10+ and works the same in Windows PowerShell 5.1 and pwsh 7).
    $text = (& curl.exe -s -N --max-time 2 "$Base/episodes/$ep/events?after=$cursor" 2>$null) -join "`n"
    foreach ($line in $text -split "`n") {
        if ($line -like 'id:*') { $cursor = [int]$line.Substring(3).Trim() }
        elseif ($line -like 'data:*') {
            $e = $line.Substring(5) | ConvertFrom-Json
            $p = $e.body.payload
            switch ($e.body.kind) {
                'message.delta' { }
                'message.user' { Write-Host "  [$cursor] message.user: $($p.text)" }
                'message.completed' { Write-Host "  [$cursor] message.completed: $($p.text)" }
                'episode.status_changed' { $status = $p.to; Write-Host "  [$cursor] status -> $($p.to)  $($p.reason)" }
                'approval.requested' {
                    Write-Host "  [$cursor] approval requested: $($p.action) -> $Decision"
                    Post "/episodes/$ep/approvals/$($p.approval_id)" @{ decision = $Decision } | Out-Null
                }
                'tool.started' { Write-Host "  [$cursor] tool: $($p.tool)" }
                default { Write-Host "  [$cursor] $($e.body.kind)" }
            }
        }
    }
    switch ($status) {
        'completed' { Write-Host 'OK: the turn completed'; exit 0 }
        { $_ -in 'failed', 'cancelled', 'waiting_user' } { [Console]::Error.WriteLine("turn ended as $status"); exit 1 }
    }
}
[Console]::Error.WriteLine("timed out after ${TimeoutSeconds}s (last status: $status)")
exit 1
