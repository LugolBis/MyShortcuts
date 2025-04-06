BASH_SCRIPT = "$(PWD)/shell_script/main.sh"
POWERSHELL_SCRIPT =  "$(PWD)\\shell_script\\main.ps1"

ifeq ($(OS),Windows_NT)
	CONFIG_RULE="config_powershell"
else
	CONFIG_RULE="config_bash"
endif

"": config

config:
	@$(MAKE) $(CONFIG_RULE)

config_powershell:
	@cp $(POWERSHELL_SCRIPT)

config_bash:
	@chmod +x $(BASH_SCRIPT)
	@echo "" > current_command.txt

clear:
	@echo "" > current_command.txt
	@rm my_shortcuts.db