package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/sdl"
)

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
	defer destroyDisplay()
	initDisplay()

	running := true

	for running {
		running = processInputs()

		clearFrame()
		presentFrame()

	}

	sdl.Quit()
}

type object struct {
	t_id string
}

type tile struct {
	t_id string
}
