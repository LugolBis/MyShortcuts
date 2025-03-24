SCRIPT_PATH = "$(PWD)/main.sh"

"": config run

config:
	@chmod +x main.sh
	@echo "" > current_command.txt

run:
	x-terminal-emulator -e $(SCRIPT_PATH) "$(PWD)/current_command.txt"

clear:
	@echo "" > current_command.txt
	@rm my_shortcuts.db