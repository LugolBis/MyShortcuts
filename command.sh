{
    USED_INDEX=$(tmux list-windows -t myshortcuts -F "#{window_index}" 2>/dev/null)
    NEW_INDEX=1
    while echo "$USED_INDEX" | grep -q -w "$NEW_INDEX"; do
        NEW_INDEX=$(($NEW_INDEX + 1))
    done

    CURRENT_COMMAND=$(cat current_command.txt)
    tmux new-window -t myshortcuts
    tmux send-keys -t myshortcuts:$NEW_INDEX "$CURRENT_COMMAND" C-m
} > log.txt 2>&1