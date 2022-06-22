package main

import (
	"log"
	"net/http"
	"net/url"
	"os"

	"github.com/joho/godotenv"

	"github.com/gin-gonic/contrib/static"
	"github.com/gin-gonic/gin"

	docs "api/docs"
	"api/models"

	swaggerfiles "github.com/swaggo/files"
	ginSwagger "github.com/swaggo/gin-swagger"
	// "api/models"
)

// @Summary Example main handler
// @Schemes
// @Description Some really interesting description
// @Accept json
// @Produce json
// @Router / [get]
func MainHandler(c *gin.Context) {
	c.JSON(200, gin.H{"data": "example"})
}

// @title           Swagger for Deps Graph
// @version         1.0
// @description     Developing with swagger is easier, please document new endpoints!
// @termsOfService  https://github.com/TDiblik/deps-graph/blob/master/LICENSE

// @contact.name   API Support
// @contact.url    https://www.youtube.com/watch?v=dQw4w9WgXcQ
// @contact.email  dibla.tomas@post.cz

// @host      localhost:8000
func main() {

	// load .env file
	errEnvLoad := godotenv.Load(".env")
	if errEnvLoad != nil {
		log.Fatalf("Error loading .env file")
		return
	}

	// Config server
	if os.Getenv("GIN_DEBUG") != "true" {
		gin.SetMode(gin.ReleaseMode)
	}

	// TODO: Gin has this big error message about proxies, look into it and set it up accordingly.

	// Setup db
	models.Setup()

	// Default server config
	r := gin.Default()

	// Serve react
	r.Use(static.Serve("/", static.LocalFile("./react-output", true)))

	// App routes
	v1 := r.Group("/api/v1")
	{
		v1.GET("/", MainHandler) // Placeholder
		// v1.GET("/books", controllers.FindBooks)
		// v1.POST("/books", controllers.CreateBook)
	}

	// Setup swagger
	docs.SwaggerInfo.BasePath = "/api/v1"
	r.GET("/docs/", func(c *gin.Context) {
		location := url.URL{Path: "/swagger/index.html"}
		c.Redirect(http.StatusMovedPermanently, location.RequestURI())
	})
	r.GET("/swagger/*any", ginSwagger.WrapHandler(swaggerfiles.Handler))

	// Start the server
	r.Run(":8000")
}
