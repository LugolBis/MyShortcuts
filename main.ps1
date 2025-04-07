$fileCommande = "$(Get-Location)\current_command.txt"
$command = Get-Content $fileCommande -Raw
Start-Process powershell.exe -ArgumentList "-NoExit", "-Command", $command