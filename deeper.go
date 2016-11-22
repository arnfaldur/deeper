package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/sdl"
)

const (
	MAPSIZE = 16
)

type Mapt [MAPSIZE][MAPSIZE]Tile

var renderables []*Actor
var themap Mapt
var hilbert Player
var actors []NPC

func temp_addDummy(xpos, ypos int) {
	actors = append(actors, dummyNPC(xpos, ypos))
}

func temp_populatemap() {
	fmt.Println("STONE_FLOOR: %v", STONE_FLOOR)
	for i := 0; i < MAPSIZE; i++ {
		for j := 0; j < MAPSIZE; j++ {
			if i != 0 && j != 0 && i != MAPSIZE-1 && j != MAPSIZE-1 {
				themap[i][j] = Tile{tileID: STONE_FLOOR}
			} else {
				themap[i][j] = Tile{tileID: STONE_WALL}
			}
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

	hilbert = Player{Entity{x: MAPSIZE / 2, y: MAPSIZE / 2, damage: 5}, PLAYER}
	temp_populatemap()
	term_rendermap()

	for running {
		//running = processInputs()
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

	//Start hack:

	hilbert = Player{Entity{x: 3, y: 3, damage: 5}, PLAYER}
	temp_populatemap()

	for running {
		//update_key_state()
		running = !get_key_state(sdl.SCANCODE_ESCAPE)

		//fmt.Println("ESCAPE: ", sdl.SCANCODE_ESCAPE)
		//fmt.Println("UP: ", sdl.SCANCODE_UP)
		//fmt.Println("LEFT: ", sdl.SCANCODE_LEFT)

		if get_key_state(sdl.SCANCODE_UP) {
			hilbert.termupdate(&themap, &actors, UP)
		}

		if get_key_state(sdl.SCANCODE_DOWN) {
			hilbert.termupdate(&themap, &actors, DOWN)
		}

		if get_key_state(sdl.SCANCODE_LEFT) {
			hilbert.termupdate(&themap, &actors, LEFT)
		}

		if get_key_state(sdl.SCANCODE_RIGHT) {
			hilbert.termupdate(&themap, &actors, RIGHT)
		}

		for i := 0; i < len(actors); i++ {
			if actors[i].currHealth <= 0 {
				actors = append(actors[:i], actors[i+1:]...)
			}
		}

		clearFrame()
		renderMap(&themap, &actors, &hilbert)
		presentFrame()
		//term_rendermap()
	}
	//End hack;
	/*
		for running {
			running = processInputs()

			clearFrame()
			renderMap()
			presentFrame()

		}
	*/
	unloadTextures()
	sdl.Quit()
}

type object struct {
	t_id string
}

type tile struct {
	t_id string
}
