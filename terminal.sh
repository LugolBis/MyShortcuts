#!/bin/bash

SESSION_NAME="myshortcuts"

# Assert that MYSHORTCUTS_DIR variable exist
if [ -z "$MYSHORTCUTS_DIR" ]; then
    echo "Error :  the environement variable MYSHORTCUTS_DIR isn't set" >&2
    exit 1
fi

# Assert that the directory exist
if [ ! -d "$MYSHORTCUTS_DIR" ]; then
    echo "Error : the directory $MYSHORTCUTS_DIR doesn't exist" >&2
    exit 1
fi

# Create the session if necessary and send command
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    WINDOW_INDEX="0"
    if tmux list-windows -t "$SESSION_NAME" 2>/dev/null | grep -q "^\s*$WINDOW_INDEX:"; then
        echo ""
    else
        tmux new-window -t myshortcuts
        tmux send-keys -t "${SESSION_NAME}:0.0" "cd $MYSHORTCUTS_DIR && cargo run" C-m
    fi
else
    tmux new-session -d -s "$SESSION_NAME" -c "$MYSHORTCUTS_DIR"
    tmux send-keys -t "${SESSION_NAME}:0.0" "cargo run" C-m
fi

if [ -z "$TMUX" ]; then
    tmux attach -t "$SESSION_NAME"
else
    tmux switch-client -t "$SESSION_NAME"
fi

# To create a new window :
# tmux new-window -t myshortcuts
# To send a command to a window :
# tmux send-keys -t myshortcuts:1 "echo Hello Window 1 !" C-m