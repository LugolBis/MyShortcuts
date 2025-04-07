config_windows:
	echo "Not yet implemented."

config_bash:
	@chmod +x main.sh
	@chmod +x current_command.txt

clear:
	@echo "" > config.txt
	@echo "" > current_command.txt
	@rm my_shortcuts.db