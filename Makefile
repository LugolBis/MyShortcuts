.PHONY: bash_set bash_unset zsh zsh_set zsh_unset clear

ifeq ($(UNAME_S),Darwin)
SED_INPLACE := sed -i ''
else
SED_INPLACE := sed -i
endif

bash:
	@$(MAKE) bash_unset ALIAS=myshortcuts
	@$(MAKE) bash_set ALIAS=myshortcuts COMMAND='export MYSHORTCUTS_DIR=$(PWD) && $(PWD)/terminal.sh'
	@chmod +x terminal.sh
	@chmod +x command.sh

bash_set:
	@if [ -z "$(ALIAS)" ] || [ -z "$(COMMAND)" ]; then \
		echo "ERROR: You need to specify ALIAS et COMMAND"; \
		echo "Usage: make bash_set ALIAS=<nom_alias> COMMAND='<commande>'"; \
		exit 1; \
	fi
	@if ! grep -q "\nalias $(ALIAS)=" ~/.bashrc; then \
		echo "\nalias $(ALIAS)='$(COMMAND)'" >> ~/.bashrc; \
		echo "Alias '$(ALIAS)' added in ~/.bashrc"; \
		echo "Execute 'source ~/.bashrc' to activate it"; \
	else \
		echo "Alias '$(ALIAS)' already exist in ~/.bashrc"; \
	fi

bash_unset:
	@if [ -z "$(ALIAS)" ]; then \
		echo "ERROR: You need to specify ALIAS"; \
		echo "Usage: make bash_unset ALIAS=<nom_alias>"; \
		exit 1; \
	fi
	@if grep -q "alias $(ALIAS)=" ~/.bashrc; then \
		sed -i "/alias $(ALIAS)=/d" ~/.bashrc; \
		echo "Alias '$(ALIAS)' supprimé de ~/.bashrc"; \
		echo "Execute 'source ~/.bashrc' to apply changes"; \
	else \
		echo "Can't find alias '$(ALIAS)' in ~/.bashrc"; \
	fi

zsh:
	@$(MAKE) zsh_unset ALIAS=myshortcuts
	@$(MAKE) zsh_set ALIAS=myshortcuts COMMAND='export MYSHORTCUTS_DIR=$(PWD) && $(PWD)/terminal.sh'
	@chmod +x terminal.sh
	@chmod +x command.sh

zsh_set:
	@if [ -z "$(ALIAS)" ]; then \
		echo "Errot: ALIAS not defined"; \
		exit 1; \
	fi
	@if [ -z "$(COMMAND)" ]; then \
		echo "Error: COMMAND not defined"; \
		exit 1; \
	else \
		$(SED_INPLACE) '/^alias $(ALIAS)=/d' ~/.zshrc; \
		echo "alias $(ALIAS)='$(COMMAND)'" >> ~/.zshrc; \
		@echo "Alias $(ALIAS) seted. Execute 'source ~/.zshrc' to apply changes"; \
	fi

zsh_unset:
	@if [ -z "$(ALIAS)" ]; then \
		echo "Error: ALIAS not defined."; \
		exit 1; \
	else \
		$(SED_INPLACE) '/^alias $(ALIAS)=/d' ~/.zshrc; \
		@echo "Alias $(ALIAS) deleted. Execute 'source ~/.zshrc' to apply changes"; \
	fi

clear:
	@echo "" > current_command.txt
	@rm my_shortcuts.db