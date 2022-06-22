package controllers

// import (
// 	"net/http"

// 	"api/models"

// 	"github.com/gin-gonic/gin"
// )

// @Summary Example db conenction
// @Schemes
// @Description Some really interesting description
// @Accept json
// @Produce json
// @Router /books [get]
// func FindBooks(c *gin.Context) {
// 	var books []models.Book
// 	models.DB.Find(&books)

// 	c.JSON(http.StatusOK, gin.H{"data": books})
// }

// type CreateBookInput struct {
// 	Title  string `json:"title" binding:"required"`
// 	Author string `json:"author" binding:"required"`
// }

// @Summary Example insert into db
// @Schemes
// @Description Some really interesting description
// @Accept json
// @Produce json
// @Param title body string true "Title"
// @Success 200 {object} models.Book
// @Router /books/create [post]
// func CreateBook(c *gin.Context) {
// 	// Validate input
// 	var input CreateBookInput
// 	if err := c.ShouldBindJSON(&input); err != nil {
// 		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
// 		return
// 	}

// 	// Create book
// 	book := models.Book{Title: input.Title, Author: input.Author}
// 	models.DB.Create(&book)

// 	c.JSON(http.StatusOK, gin.H{"data": book})
// }
