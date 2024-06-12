
use builtin;
use str;

set edit:completion:arg-completer[powernotd] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'powernotd'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'powernotd'= {
            cand -f 'Set config-file path if needed, otherwise $XDG_CONFIG_HOME/powernotd/config.json is used'
            cand --config-file 'Set config-file path if needed, otherwise $XDG_CONFIG_HOME/powernotd/config.json is used'
            cand -b 'Pass the battery such as ''BAT1'' if your system has multiple and you do not want to use the default (BAT0). Check ''/sys/class/power_supply/'' to see which batteries you have'
            cand --battery 'Pass the battery such as ''BAT1'' if your system has multiple and you do not want to use the default (BAT0). Check ''/sys/class/power_supply/'' to see which batteries you have'
            cand -s 'Print the current battery-level to stdout then exit'
            cand --status-level 'Print the current battery-level to stdout then exit'
            cand -c 'Print charging status ''charging'', ''discharging'', ''full'' or ''unknown'' to stdout then exit'
            cand --charging-state 'Print charging status ''charging'', ''discharging'', ''full'' or ''unknown'' to stdout then exit'
            cand -n 'Send desktop notification with current battery-level then exit'
            cand --notify-now 'Send desktop notification with current battery-level then exit'
            cand -t 'List all notification thresholds in the format ''a_1%, a_2%, ..., a_n%'' that are specified in the config-file'
            cand --list-thresholds 'List all notification thresholds in the format ''a_1%, a_2%, ..., a_n%'' that are specified in the config-file'
            cand -p 'Display the path to the config-file'
            cand --show-config-path 'Display the path to the config-file'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
