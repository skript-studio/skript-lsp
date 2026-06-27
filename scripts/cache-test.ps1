$ErrorActionPreference = "Stop"
$bin = "D:\Documents\Github Projects\skript-lsp-rust\target\release\skript-lsp.exe"
$cacheDir = "D:\Documents\Github Projects\skript-lsp-rust\extensions\vscode-skript\data"

function fmt($json) {
  $b = [Text.Encoding]::UTF8.GetBytes($json)
  $h = [Text.Encoding]::ASCII.GetBytes("Content-Length: $($b.Length)`r`n`r`n")
  return $h + $b
}

$msgs = @(
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"capabilities":{},"rootUri":null}}',
  '{"jsonrpc":"2.0","method":"initialized","params":{}}',
  '{"jsonrpc":"2.0","id":2,"method":"shutdown"}',
  '{"jsonrpc":"2.0","method":"exit"}'
)

$all = @()
$msgs | % { $all += fmt $_ }
[System.IO.File]::WriteAllBytes("cache-stdin.bin", $all)

$proc = Start-Process -FilePath $bin -ArgumentList "--stdio","--log-level","debug","--cache-dir","$cacheDir" -NoNewWindow -PassThru -RedirectStandardInput "cache-stdin.bin" -RedirectStandardOutput "cache-stdout.txt" -RedirectStandardError "cache-stderr.txt"
Start-Sleep -Seconds 5

Write-Host "=== STDERR ==="
Get-Content "cache-stderr.txt" -Raw
Write-Host "=== STDOUT ==="
Get-Content "cache-stdout.txt" -Raw

if ($proc.HasExited) { Write-Host "Exited: $($proc.ExitCode)" }
else { $proc.Kill(); Write-Host "Killed" }

Remove-Item "cache-stdin.bin","cache-stdout.txt","cache-stderr.txt" -ErrorAction SilentlyContinue
