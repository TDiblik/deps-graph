PROJECT_NAME = api
PROJECT_DIR = ./$(PROJECT_NAME)/
BIN_FOLDER = out

CD = cd $(PROJECT_DIR) && \

DC = docker
DC_REDIS_GRAPH_NAME = deps-graph-redisGraph
DC_RG_UP = $(DC) run -it --name $(DC_REDIS_GRAPH_NAME) -d -p 6379:6379 -v $(pwd)/db-data/redisGraph:/data redislabs/redisgraph:2.8.13
DC_RG_FIND = $(DC) ps -a -q --filter="name=$(DC_REDIS_GRAPH_NAME)"
DC_RG_DOWN = $(DC) rm $(shell $(DC) stop $(shell $(DC_RG_FIND)))

DT = dotnet
DT_RESTORE = $(DT) restore
DT_DEV = $(DT) watch run
DT_DEV_HARD = $(DEV_API) --no-hot-reload
DT_PUBLISH = $(DT) publish -c Release -o $(BIN_FOLDER)

COMPILATION_GARBAGE = bin/ obj/ $(BIN_FOLDER)/

prepare: clean
	$(DC_RG_UP)
	$(CD) $(DT_RESTORE)

# Does hot reload, does not generate XML docs
# Useful for most of development
# You can also manually trigger full build using "Ctrl + R"
dev-api:
	$(CD) $(DT_DEV)

# Rebuilds project every time, generates XML docs
# Useful for swagger tinkerink
dev-api-hard:
	$(CD) $(DT_DEV_HARD)

# Production ready build
publish: 
	$(CD) $(DT_PUBLISH)
	
clean:
	if [ $(shell $(DC_RG_FIND) | wc -l) = 1 ]; then\
		$(DC_RG_DOWN);\
	fi;
	$(CD) rm -rf $(COMPILATION_GARBAGE)