# posh-fzf

[Fzf](https://github.com/junegunn/fzf) keybinding integration with powershell 5.1 and pwsh >= 7.

## Installation

Add this to `code $PROFILE`:
```powershell
Invoke-Expression (&posh-fzf init | Out-String)

Set-PSReadLineKeyHandler -Key 'Ctrl+t' -ScriptBlock { Invoke-PoshFzfSelectItems }
Set-PSReadLineKeyHandler -Key 'Alt+c' -ScriptBlock { Invoke-PoshFzfChangeDirectory }
Set-PSReadLineKeyHandler -Key 'Ctrl+r' -ScriptBlock { Invoke-PoshFzfSelectHistory }
```

## Built in commands
| Command                         | Description                                                                           |
| ------------------------------- | ------------------------------------------------------------------------------------- |
| `Invoke-PoshFzfSelectItems`     | Select one or more items and paste it to the terminal.                                |
| `Invoke-PoshFzfChangeDirectory` | Show all child directories, and if selected, `cd` to that directory.                  |
| `Invoke-PoshFzfSelectHistory`   | Show distinct historical commands in most recent order.                               |
| `Invoke-PoshFzf`                | Invokes `posh-fzf` using the StartProcess api. See custom commands below.             |
| `Invoke-PoshFzfInsertUtf8`      | Inserts text forcing UTF-8 encoding.                                                  |
| `Invoke-PoshFzfTempEnv`         | Temporarily set environment variables, and reset them after executing a script block. |

### Customisation commands
| Command                                | Description                                                                          |
| -------------------------------------- | ------------------------------------------------------------------------------------ |
| `Invoke-PoshFzfRedrawLastLineOfPrompt` | Set this to provide an alternative prompt redraw function, instead of using `prompt` |

For example, for starship multiline terminals, you can improve performance by defining a function that avoids calling `starship`:
```powershell
function Invoke-PoshFzfRedrawLastLineOfPrompt {
    Write-Output "`e[1;32m$([char]0x276F)"
}
```

## Custom commands

You can use any `fzf` command in keybindings by tweaking it to use the `Invoke-PoshFzf` cmdlet.

For example, instead of `git branch | fzf`, you would have:

```powershell
Set-PSReadLineKeyHandler -Key 'Alt+b' -ScriptBlock { 
    $branch = git branch | Invoke-PoshFzf -poshFzfArgs @("fzf")
    if ($branch) {
        [Microsoft.PowerShell.PSConsoleReadLine]::DeleteLine()
        Invoke-PoshFzfInsertUtf8("git checkout '$branch'")
        [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
    }
}
```

You can also executables other than `fzf`.

For example, [zoxide](https://github.com/ajeetdsouza/zoxide) is a smarter `cd` command, you can inspect it's db using `zoxide query -i`.  
Internally, it calls `fzf`. Instead of `$selection = (zoxide query -i)`, it can be called using the `custom` subcommand:

```powershell
Set-PSReadLineKeyHandler -Key 'Ctrl+shift+z' -ScriptBlock {
    # 45% height comes from: https://github.com/ajeetdsouza/zoxide/blob/a624ceef54a31de2d0624e9eb14ce65024cc9e79/src/cmd/query.rs#L92
    $fzfSelection = Invoke-PoshFzf -poshFzfArgs @("custom", "--", "zoxide", "query", "-i") -heightRowsOrPercent "45%"
    if ($fzfSelection) {
        [Microsoft.PowerShell.PSConsoleReadLine]::DeleteLine()
        Invoke-PoshFzfInsertUtf8("cd $fzfSelection")
        [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
    }
}
```

Importantly, `-heightRowsOrPercent` must be specified to match the `--height` argument of fzf. This is due to a limitation in PSReadLine when expanding the terminal