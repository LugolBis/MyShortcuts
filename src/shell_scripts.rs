pub const TERMINAL: &str = r#"
SESSION_NAME="myshortcuts"

# Create the session if necessary and send command
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    WINDOW_INDEX="0"
    if tmux list-windows -t "$SESSION_NAME" 2>/dev/null | grep -q "^\s*$WINDOW_INDEX:"; then
        echo ""
    else
        tmux new-window -t myshortcuts
        tmux send-keys -t "${SESSION_NAME}:0.0" "export MYSHORTCUTSLAUNCH=1 && myshortcuts" C-m
    fi
else
    tmux new-session -d -s "$SESSION_NAME"
    tmux send-keys -t "${SESSION_NAME}:0.0" "export MYSHORTCUTSLAUNCH=1 && myshortcuts" C-m
fi
if [ -z "$TMUX" ]; then
    tmux attach -t "$SESSION_NAME"
else
    tmux switch-client -t "$SESSION_NAME"
fi
"#;

pub const COMMAND: &str = r#"
{
    USED_INDEX=$(tmux list-windows -t myshortcuts -F "\#{window_index}" 2>/dev/null)
    NEW_INDEX=1
    while echo "$USED_INDEX" | grep -q -w "$NEW_INDEX"; do
        NEW_INDEX=$(($NEW_INDEX + 1))
    done

    CURRENT_COMMAND=$(cat $MYSHORTCUTS_DIR/current_command.txt)
    tmux new-window -t myshortcuts
    tmux send-keys -t myshortcuts:$NEW_INDEX "$CURRENT_COMMAND" C-m
} > log.txt 2>&1
"#;

pub const MAIN: &str = r#"
$fileCommande = "$MYSHORTCUTS_DIR\current_command.txt"
$command = Get-Content $fileCommande -Raw
Start-Process powershell.exe -ArgumentList "-NoExit", "-Command", $command
"#;