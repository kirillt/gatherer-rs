NAME ?= metrics-gatherer
DOCKER ?= podman

build:
	@$(DOCKER) build -t $(NAME) .

rebuild:
	@$(DOCKER) build --no-cache --squash -t $(NAME) .

run:
	@$(DOCKER) run --net=host -d $(NAME) /web/start.sh

inspect:
	@$(DOCKER) run --net=host -it --rm $(NAME) /bin/bash
