package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/sdl"
)

var themap [64][64]Tile

func main() {

	fmt.Println("Welcome to Deeper!")
	fmt.Println("Would you like terminal mode or graphical mode? t/g")
	var ans string
	fmt.Scan(&ans)

	if ans == "t" {
		termGameLoop()
	} else if ans == "g" {
		sdlGameLoop()
	}
}

func termGameLoop() {

	running := true

	for running {
		running = processInputs()
	}
}

func sdlGameLoop() {
	initDisplay()
	defer destroyDisplay()
	loadTextures()

	running := true

	for running {
		running = processInputs()

		clearFrame()
		renderGame()
		presentFrame()

	}

	unloadTextures()
	sdl.Quit()
}

type object struct {
	t_id string
}

type tile struct {
	t_id string
}
