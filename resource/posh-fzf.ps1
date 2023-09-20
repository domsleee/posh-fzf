$null = New-Module posh-fzf {
	$defaultHeight = "45%"
	$defaultHeightArg = "--height=$defaultHeight"

	function Invoke-PoshFzfSelectItems {
		Invoke-PoshFzfTempEnv @{FZF_DEFAULT_COMMAND = 'fd --hidden --exclude ".git"'} -ScriptBlock {
			$fzfSelection = Invoke-PoshFzfStartProcess -FileName "fzf" -Arguments @("$defaultHeightArg", "-m")
			if ($fzfSelection) {
				Invoke-PoshFzfInsertUtf8 $fzfSelection
			}
		}
	}
	
	function Invoke-PoshFzfChangeDirectory {
		Invoke-PoshFzfTempEnv @{FZF_DEFAULT_COMMAND = 'fd --type d --hidden --exclude ".git"'} -ScriptBlock {
			$directory = Invoke-PoshFzfStartProcess -FileName "fzf" -Arguments @("$defaultHeightArg", "--preview", "fd . {} --maxdepth 1")
			if ($directory) {
				[Microsoft.PowerShell.PSConsoleReadLine]::DeleteLine()
				Invoke-PoshFzfInsertUtf8 "cd $directory"
				[Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
			}
		}
	}
	
	function Invoke-PoshFzfSelectHistory {
		$historyPath = (Get-PSReadLineOption).HistorySavePath
		$historyCommand = Invoke-PoshFzfStartProcess -FileName "posh-fzf" -Arguments @("history", $historyPath)
		if ($historyCommand) {
			[Microsoft.PowerShell.PSConsoleReadLine]::DeleteLine()
			Invoke-PoshFzfInsertUtf8 $historyCommand
		}
	}
	
	# // https://github.com/kelleyma49/PSFzf/blob/051a0da959321253fd11bf9bb14e270b2902eb5c/PSFzf.Base.ps1#L258-L560
	function Invoke-PoshFzfStartProcess {
		[CmdletBinding()]
		param (
			[Parameter(ValueFromPipeline=$true)]
			[string]$inputString,
			[Parameter(Mandatory)]
			[string]$FileName,
			[array]$Arguments = @(),
			[string]$HeightRowsOrPercent = $defaultHeight
		)
		begin {
			$numRows = CalculateHeight($HeightRowsOrPercent)
			ClearBufferAhead $numRows
			$startInfo = New-Object System.Diagnostics.ProcessStartInfo
			$startInfo.FileName = $FileName
			Set-ArgumentList $startInfo $Arguments
			$startInfo.RedirectStandardInput = $MyInvocation.ExpectingInput
			$startInfo.RedirectStandardOutput = $true
			$startInfo.UseShellExecute = $false
			$startInfo.WorkingDirectory = (Get-Location).Path
			
			$process = New-Object System.Diagnostics.Process
			$process.StartInfo = $startInfo
			$process.Start() | Out-Null
		}
		process {
			if ($inputString) {
				$process.StandardInput.WriteLine($inputString)
			}
		}
		end {
			$process.WaitForExit()
			RedrawLastLineOfPrompt
			$content = $process.StandardOutput.ReadToEnd().Trim() -join "`n"
			return $content
		}
	}

	function Set-ArgumentList($startInfo, $argumentList) {
		# Taken from https://github.com/starship/starship/blob/43b2d42cd526e34c5f0290e7409fbd6d3a54e908/src/init/starship.ps1#L35-L53
		if ($startInfo.ArgumentList.Add) {
			# PowerShell 6+ supports ArgumentList property
			foreach ($arg in $argumentList) { $startInfo.ArgumentList.Add($arg) }
		}
		else {
			# Build an arguments string which follows the C++ command-line argument quoting rules
			# See: https://docs.microsoft.com/en-us/previous-versions//17w5ykft(v=vs.85)?redirectedfrom=MSDN
			$escaped = $argumentList | ForEach-Object {
				$s = $_ -Replace '(\\+)"','$1$1"'; # Escape backslash chains immediately preceding quote marks.
				$s = $s -Replace '(\\+)$','$1$1';  # Escape backslash chains immediately preceding the end of the string.
				$s = $s -Replace '"','\"';         # Escape quote marks.
				"`"$s`""                           # Quote the argument.
			}
			$startInfo.Arguments = $escaped -Join ' ';
		}
	}

	function Invoke-PoshFzfTempEnv([hashtable]$envObj, [scriptblock]$ScriptBlock) {
		$originals = @{}
		foreach ($key in $envObj.keys) {
			$originals[$key] = ""# (Get-Item "env:$key" -ErrorAction SilentlyContinue).Value
			Set-Item "env:$key" $envObj[$key]

		}
		try {
			Invoke-Command -ScriptBlock $ScriptBlock
		} finally {
			foreach ($key in $originals.keys) {
				Set-Item "env:$key" $originals[$key]
			}
		}
	}

	function ClearBufferAhead([int] $numRows) {
		# FIXME: the bell still fires
		$originalBellStyle = (Get-PSReadLineOption).BellStyle
		Set-PSReadLineOption -BellStyle None
		[Microsoft.PowerShell.PSConsoleReadLine]::Insert("`n" * ($numRows))
		[Microsoft.PowerShell.PSConsoleReadLine]::Undo()
		Set-PSReadLineOption -BellStyle $originalBellStyle
	}

	function RedrawLastLineOfPrompt {
		$previousOutputEncoding = [Console]::OutputEncoding
		[Console]::OutputEncoding = [Text.Encoding]::UTF8

		try {
			$prompt = ""
			if (Get-Command "Invoke-PoshFzfRedrawLastLineOfPrompt" -ErrorAction SilentlyContinue) {
				$prompt = Invoke-PoshFzfRedrawLastLineOfPrompt
			} else {
				$prompt = prompt 6>$null
			}
			$lastLine = ($prompt -split "`n")[-1]
			Write-Host -NoNewline $lastLine
			[Microsoft.PowerShell.PSConsoleReadLine]::Insert(" ")
			[Microsoft.PowerShell.PSConsoleReadLine]::Undo()
		} finally {
			[Console]::OutputEncoding = $previousOutputEncoding
		}
	}
	
	function Invoke-PoshFzfInsertUtf8($data) {
		$previousOutputEncoding = [Console]::OutputEncoding
		[Console]::OutputEncoding = [Text.Encoding]::UTF8
		[Microsoft.PowerShell.PSConsoleReadLine]::Insert($data)
		[Console]::OutputEncoding = $previousOutputEncoding
	}
	
	function CalculateHeight($heightRowsOrPercent) {
		if ($heightRowsOrPercent -is [int]) {
			return $heightRowsOrPercent
		}
		# https://github.com/junegunn/fzf/blob/d2b852f7cbd2da53804de00cf22ca0b7e6c9f472/src/terminal.go#L547-L552
		$heightPercent = [int]$heightRowsOrPercent.TrimEnd('%')
		$termHeight = $host.UI.RawUI.WindowSize.Height
		$minHeight = 10 # 10 is the default
		return [Math]::Max([int]($heightPercent * [double]$termHeight / 100.0), $minHeight)
	}
	
	Export-ModuleMember -Function @(
		"Invoke-PoshFzfSelectItems",
		"Invoke-PoshFzfChangeDirectory",
		"Invoke-PoshFzfSelectHistory",
		"Invoke-PoshFzfStartProcess",
		"Invoke-PoshFzfInsertUtf8",
		"Invoke-PoshFzfTempEnv"
	)
}
