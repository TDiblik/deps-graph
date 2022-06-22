package models

import (
	"os"
	"strconv"
	"time"

	"github.com/gomodule/redigo/redis"
)

var RedisConn *redis.Pool

// Setup Initialize the Redis instance
func Setup() error {
	MaxIdle, errMaxIdle := strconv.Atoi(os.Getenv("REDIS_MAX_IDLE"))
	MaxActive, errMaxActive := strconv.Atoi(os.Getenv("REDIS_MAX_ACTIVE"))
	IdleTimeout, errIdleTimeout := strconv.Atoi(os.Getenv("REDIS_IDLE_TIMEOUT"))
	if errMaxIdle != nil || errIdleTimeout != nil || errMaxActive != nil {
		panic("Input env variables for database")
	}

	RedisConn = &redis.Pool{
		MaxIdle:     MaxIdle,
		MaxActive:   MaxActive,
		IdleTimeout: time.Duration(IdleTimeout),
		Dial: func() (redis.Conn, error) {
			c, err := redis.Dial("tcp", os.Getenv("REDIS_HOST"))
			if err != nil {
				return nil, err
			}
			if os.Getenv("REDIS_PASSWORD") != "" {
				if _, err := c.Do("AUTH", os.Getenv("REDIS_PASSWORD")); err != nil {
					c.Close()
					return nil, err
				}
			}
			return c, err
		},
		TestOnBorrow: func(c redis.Conn, t time.Time) error {
			_, err := c.Do("PING")
			return err
		},
	}

	return nil
}
