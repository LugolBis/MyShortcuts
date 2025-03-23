SCRIPT_PATH = "$(PWD)/main.sh"

"": config run

config:
	@chmod +x main.sh
	@echo "" > current_command.sh
	@chmod +x current_command.sh

run:
	x-terminal-emulator -e $(SCRIPT_PATH) "$(PWD)/current_command.sh"

clear:
	@echo "" > current_command.sh