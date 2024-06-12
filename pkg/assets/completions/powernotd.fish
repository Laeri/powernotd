complete -c powernotd -s f -l config-file -d 'Set config-file path if needed, otherwise $XDG_CONFIG_HOME/powernotd/config.json is used' -r
complete -c powernotd -s b -l battery -d 'Pass the battery such as \'BAT1\' if your system has multiple and you do not want to use the default (BAT0). Check \'/sys/class/power_supply/\' to see which batteries you have' -r
complete -c powernotd -s s -l status-level -d 'Print the current battery-level to stdout then exit'
complete -c powernotd -s c -l charging-state -d 'Print charging status \'charging\', \'discharging\', \'full\' or \'unknown\' to stdout then exit'
complete -c powernotd -s n -l notify-now -d 'Send desktop notification with current battery-level then exit'
complete -c powernotd -s t -l list-thresholds -d 'List all notification thresholds in the format \'a_1%, a_2%, ..., a_n%\' that are specified in the config-file'
complete -c powernotd -s p -l show-config-path -d 'Display the path to the config-file'
complete -c powernotd -s h -l help -d 'Print help'
complete -c powernotd -s V -l version -d 'Print version'
