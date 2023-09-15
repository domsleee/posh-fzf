
Set-PSReadLineKeyHandler -Key 'Ctrl+t' -ScriptBlock {
	$fzfSelection = Invoke-Fzf @("select-multiple")
	if ($fzfSelection) {
		InsertUtf8 $fzfSelection
	}
}

Set-PSReadLineKeyHandler -Key 'Alt+c' -ScriptBlock {
	$fzfSelection = Invoke-Fzf @("change-directory")
	if ($fzfSelection) {
		[Microsoft.PowerShell.PSConsoleReadLine]::DeleteLine()
		InsertUtf8 $fzfSelection
		[Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
	}
}

Set-PSReadLineKeyHandler -Key 'Ctrl+r' -ScriptBlock {
	$historyPath = (Get-PSReadLineOption).HistorySavePath
	$fzfSelection = Invoke-Fzf @("history", $historyPath)
	if ($fzfSelection) {
		[Microsoft.PowerShell.PSConsoleReadLine]::DeleteLine()
		InsertUtf8 $fzfSelection
	}
}

# // https://github.com/kelleyma49/PSFzf/blob/051a0da959321253fd11bf9bb14e270b2902eb5c/PSFzf.Base.ps1#L258-L560
function Invoke-Fzf([array]$poshFzfArgs) {
	$numRows = if ($env:POSH_FZF_NUM_ROWS) { $env:POSH_FZF_NUM_ROWS } else { 11 }
	[Microsoft.PowerShell.PSConsoleReadLine]::Insert("`n" * ($numRows))
	[Microsoft.PowerShell.PSConsoleReadLine]::Undo()
	$tempFileName = [System.IO.Path]::GetTempFileName()
	$argumentList = @("--temp-file-name", $tempFileName) + $poshFzfArgs
	$process = Start-Process -FilePath "posh-fzf" -ArgumentList $argumentList -PassThru -NoNewWindow
	$process.WaitForExit()
	ContinuationPrompt
	$content = Get-Content -Path $tempFileName
	return $content
}

function ContinuationPrompt {
	$previousOutputEncoding = [Console]::OutputEncoding
	[Console]::OutputEncoding = [Text.Encoding]::UTF8
	
	try {
		# $data = ((global:prompt) -split "`n")[-1]
		Write-Host -NoNewline (&starship module character)
		[Microsoft.PowerShell.PSConsoleReadLine]::Insert(" ")
		[Microsoft.PowerShell.PSConsoleReadLine]::Undo()
	} finally {
		[Console]::OutputEncoding = $previousOutputEncoding
	}
}

function InsertUtf8($data) {
	$previousOutputEncoding = [Console]::OutputEncoding
	[Console]::OutputEncoding = [Text.Encoding]::UTF8
	[Microsoft.PowerShell.PSConsoleReadLine]::Insert($data)
	[Console]::OutputEncoding = $previousOutputEncoding

}
