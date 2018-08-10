package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/sdl"
	"math/rand"
	"time"
)

const (
	MAPSIZE     int           = 64
	DURPERFRAME time.Duration = 16666666
)

type object struct {
	t_id string
}

type tile struct {
	t_id string
}

type Mapt [MAPSIZE][MAPSIZE]Tile

var renderables []*Actor
var themap Mapt
var hilbert Player
var actors []NPC

func temp_addDummy(xpos, ypos int) {
	actors = append(actors, dummyNPC(xpos, ypos))
}

func temp_populatemap() {
	fmt.Printf("STONE_FLOOR: %v\n", STONE_FLOOR)
	for y := 0; y < MAPSIZE; y++ {
		for x := 0; x < MAPSIZE; x++ {
			//true at edges and random points, for flavour, RNG is deterministic unless seeded.
			randomN := rand.Float64()
			if y == 0 || x == 0 || y == MAPSIZE-1 || x == MAPSIZE-1 || randomN > 0.8 {
				themap[y][x] = Tile{tileID: STONE_WALL}
			} else {
				if randomN > 0.3 {
					actors = append(actors, testEnemyNPC(x, y, rand.Intn(10)))
				}
				themap[y][x] = Tile{tileID: STONE_FLOOR}
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
	fmt.Println("GG!")
}
func sdlGameLoop() {
	initDisplay()
	defer destroyDisplay()
	loadTextures()

	running := true
	var event sdl.Event
	//Start hack:

	hilbert = Player{Entity{x: 3, y: 3, damage: 5}, PLAYER}
	temp_populatemap()

	//End hack

	//var stepDelay int = 0

	for running {
		var startTime time.Time = time.Now()
		//update_key_state()
		running = !get_key_state(sdl.SCANCODE_ESCAPE)

		for event = sdl.PollEvent(); event != nil; event = sdl.PollEvent() {
			switch t := event.(type) {
			case *sdl.QuitEvent:
				running = false
			case *sdl.MouseMotionEvent:
			case *sdl.MouseButtonEvent:
			case *sdl.MouseWheelEvent:
			case *sdl.KeyboardEvent:
				if t.State == sdl.PRESSED {
					if t.Keysym.Sym == sdl.K_UP {
						hilbert.termupdate(&themap, &actors, UP)
					}
					if t.Keysym.Sym == sdl.K_DOWN {
						hilbert.termupdate(&themap, &actors, DOWN)
					}
					if t.Keysym.Sym == sdl.K_LEFT {
						hilbert.termupdate(&themap, &actors, LEFT)
					}
					if t.Keysym.Sym == sdl.K_RIGHT {
						hilbert.termupdate(&themap, &actors, RIGHT)
					}
				}
			case *sdl.JoyAxisEvent:
			case *sdl.JoyBallEvent:
			case *sdl.JoyButtonEvent:
			case *sdl.JoyHatEvent:
			default:
				fmt.Printf("Some event\n")
			}

			for i := 0; i < len(actors); i++ {
				if actors[i].currHealth <= 0 {
					actors = append(actors[:i], actors[i+1:]...)
				}
			}

			clearFrame()
			renderMap(&themap, &actors, &hilbert)
			presentFrame()
		}
		fmt.Println(time.Until(startTime.Add(DURPERFRAME)))
		time.Sleep(time.Until(startTime.Add(DURPERFRAME)))
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
