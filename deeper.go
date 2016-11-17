package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/sdl"
)

const (
	MAPSIZE = 16
)

var renderables []*Actor
var themap [MAPSIZE][MAPSIZE]Tile
var hilbert Player
var actors []NPC

func temp_addDummy(xpos, ypos int) {
	actors = append(actors, dummyNPC(xpos, ypos))
}

func temp_populatemap() {
	fmt.Println("STONE_FLOOR: %v", STONE_FLOOR)
	for i := 0; i < MAPSIZE; i++ {
		for j := 0; j < MAPSIZE; j++ {
			themap[i][j] = Tile{STONE_FLOOR, 0}
		}
	}
}

func term_rendermap() {
	var printmap [MAPSIZE][MAPSIZE]string
	for i := 0; i < MAPSIZE; i++ {
		for j := 0; j < MAPSIZE; j++ {
			switch themap[i][j].tileID {
			case STONE_WALL:
				printmap[i][j] = "#"
			case STONE_FLOOR:
				printmap[i][j] = "_"
			default:
				printmap[i][j] = "?"
			}
		}
	}
	for i := 0; i < len(actors); i++ {
		printmap[actors[i].x][actors[i].y] = "*"
	}
	printmap[hilbert.x][hilbert.y] = "@"

	for i := 0; i < MAPSIZE; i++ {
		for j := 0; j < MAPSIZE; j++ {
			fmt.Print(printmap[j][i])
		}
		fmt.Print("\n")
	}

}

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

	hilbert = Player{Entity{x: MAPSIZE / 2, y: MAPSIZE / 2, damage: 5}}
	temp_populatemap()

	term_rendermap()

	for running {
		var input string
		fmt.Scan(&input)

		switch input {
		case "w":
			hilbert.termupdate(&themap, &actors, UP)
		case "s":
			hilbert.termupdate(&themap, &actors, DOWN)
		case "a":
			hilbert.termupdate(&themap, &actors, LEFT)
		case "d":
			hilbert.termupdate(&themap, &actors, RIGHT)
		case "e":
			var xpos, ypos int
			fmt.Scan(&xpos)
			fmt.Scan(&ypos)
			temp_addDummy(xpos, ypos)
		case "x":
			running = false
		}

		for i := 0; i < len(actors); i++ {
			if actors[i].currHealth <= 0 {
				actors = append(actors[:i], actors[i+1:]...)
			}
		}

		term_rendermap()
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
