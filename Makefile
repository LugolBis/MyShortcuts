SCRIPT_PATH = "$(PWD)/shell_script/main.sh"

"": config

config:
	@chmod +x $(SCRIPT_PATH)
	@echo "" > current_command.txt

clear:
	@echo "" > current_command.txt
	@rm my_shortcuts.db