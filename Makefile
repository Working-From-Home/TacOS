
define ALL_HELP_INFO
########################################################
# A simple Makefile to help with common tasks.         #
########################################################
#
#
# Global targets:
# 
#   post-create              # run the post-create.sh of the devcontainer
#   build                    # build TacOS
endef

.PHONY: all
all: help

.PHONY: help
help:
	${ALL_HELP_INFO}

.PHONY: setup
post-create:
	.devcontainer/scripts/post-create.sh

.PHONY: build
build:
	@echo "nothing yet"
