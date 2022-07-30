PROJECT_NAME = api
PROJECT_DIR = ./$(PROJECT_NAME)/

CD = cd $(PROJECT_DIR) && \

DC = docker
DC_RG_NAME = deps-graph-redisGraph
DC_RG_IMAGE = redislabs/redisgraph:2.8.13
DC_RG_UP = $(DC) run -it --name $(DC_RG_NAME) -d -p 7500:6379 -v $(shell pwd)/db-data/redisGraph:/data $(DC_RG_IMAGE)
DC_RG_FIND = $(DC) ps -a -q --filter="name=$(DC_RG_NAME)"
DC_RG_DOWN = $(DC) rm $(shell $(DC) stop $(shell $(DC_RG_FIND)))

DT = dotnet
DT_DEV = $(DT) watch run

COMPILATION_GARBAGE = bin/ obj/

all: clean build

# Production ready build
build:
	$(CD) $(DT) publish -c Release -o ./bin/out -f net6.0 --force

# Does hot reload, does not generate XML docs. You can also manually trigger full build (+ XML docs) using "Ctrl + R"
dev: dev-env
	$(CD) $(DT_DEV)

# Rebuilds project every time, generates XML docs
dev-hard: dev-env
	$(CD) $(DT_DEV) --no-hot-reload

dev-env:
	$(DC) pull $(DC_RG_IMAGE)
	if [ $(shell $(DC_RG_FIND) | wc -l) = 1 ]; then\
		$(DC_RG_DOWN);\
	fi;
	$(DC_RG_UP)

clean:
	$(CD) rm -rf $(COMPILATION_GARBAGE)
