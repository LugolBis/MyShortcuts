.PHONY: set unset clear

"":
	@$(MAKE) unset ALIAS=myshortcuts
	@$(MAKE) set ALIAS=myshortcuts COMMAND='export MYSHORTCUTS_DIR=$(PWD) && $(PWD)/terminal.sh'
	@chmod +x terminal.sh

set:
	@if [ -z "$(ALIAS)" ] || [ -z "$(COMMAND)" ]; then \
		echo "ERROR: You need to specify ALIAS et COMMAND"; \
		echo "Usage: make set ALIAS=<nom_alias> COMMAND='<commande>'"; \
		exit 1; \
	fi
	@if ! grep -q "\nalias $(ALIAS)=" ~/.bashrc; then \
		echo "\nalias $(ALIAS)='$(COMMAND)'" >> ~/.bashrc; \
		echo "Alias '$(ALIAS)' added in ~/.bashrc"; \
		echo "Execute 'source ~/.bashrc' to activate it"; \
	else \
		echo "Alias '$(ALIAS)' already exist in ~/.bashrc"; \
	fi

unset:
	@if [ -z "$(ALIAS)" ]; then \
		echo "ERROR: You need to specify ALIAS"; \
		echo "Usage: make unset ALIAS=<nom_alias>"; \
		exit 1; \
	fi
	@if grep -q "alias $(ALIAS)=" ~/.bashrc; then \
		sed -i "/alias $(ALIAS)=/d" ~/.bashrc; \
		echo "Alias '$(ALIAS)' supprimé de ~/.bashrc"; \
		echo "Execute 'source ~/.bashrc' to apply changes"; \
	else \
		echo "Can't find alias '$(ALIAS)' in ~/.bashrc"; \
	fi

clear:
	@echo "" > config.txt
	@echo "" > current_command.txt
	@rm my_shortcuts.db