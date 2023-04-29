
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'powernotd' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'powernotd'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'powernotd' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'Set config file path if needed, otherwise $XDG_CONFIG_HOME/powernotd/config.json is used')
            [CompletionResult]::new('--config-file', 'config-file', [CompletionResultType]::ParameterName, 'Set config file path if needed, otherwise $XDG_CONFIG_HOME/powernotd/config.json is used')
            [CompletionResult]::new('-s', 's', [CompletionResultType]::ParameterName, 'Print the current battery level to stdout then exit')
            [CompletionResult]::new('--status-level', 'status-level', [CompletionResultType]::ParameterName, 'Print the current battery level to stdout then exit')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Print charging status ''charging'', ''discharging'', ''full'' or ''unknown'' to stdout then exit')
            [CompletionResult]::new('--charging-state', 'charging-state', [CompletionResultType]::ParameterName, 'Print charging status ''charging'', ''discharging'', ''full'' or ''unknown'' to stdout then exit')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Send desktop notification with current battery level then exit')
            [CompletionResult]::new('--notify-now', 'notify-now', [CompletionResultType]::ParameterName, 'Send desktop notification with current battery level then exit')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
