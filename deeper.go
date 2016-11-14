package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/sdl"
)

func main() {
	fmt.Println("All is right with the world.")

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
