package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/sdl"
	"math/rand"
	"time"
)

var (
	DEBUGLOGGING = false
	HOTLOADING   = true
)

const (
	MAPSIZE     int           = 64
	DURPERFRAME time.Duration = 16666666 * 1
)

type Mapt [MAPSIZE][MAPSIZE]Entity

var theMap Mapt
var hilbert Player
var actors []NPC
var environment []*Entity

var timeDilation = 0.0

func populateMap() {
	//fmt.Printf("STONE_FLOOR: %v\n", STONE_FLOOR)
	for y := 0; y < MAPSIZE; y++ {
		for x := 0; x < MAPSIZE; x++ {
			//true at edges and random points, for flavour, RNG is deterministic unless seeded.
			randomN := rand.Float64()
			if y == 0 || x == 0 || y == MAPSIZE-1 || x == MAPSIZE-1 || randomN > 0.8 {
				theMap[y][x] = NewTile(STONE_WALL, float64(x), float64(y))
				environment = append(environment, &theMap[y][x])
			} else {
				if randomN > 0.6 {
					actors = append(actors, testEnemyNPC(complex(float64(x), float64(y)), rand.Intn(10)))
				}
				theMap[y][x] = NewTile(STONE_FLOOR, float64(x), float64(y))
				environment = append(environment, &theMap[y][x])
			}
		}
	}
}

func main() {
	initDisplay()
	defer destroyDisplay()
	loadTextures()

	running := true
	var event sdl.Event
	var pressedKeys [512]bool

	hilbert = Player{Character{Entity: Entity{id: PLAYER, pos: 3 + 3i, size: 0.8}, damage: 5}}
	populateMap()

	//var stepDelay int = 0

	for running {
		var startTime = time.Now()

		if time.Now().Sub(startTime).Nanoseconds() > time.Millisecond.Nanoseconds()*10 {
			fmt.Println("Hotloader hang!")
		}

		// Input handling

		for event = sdl.PollEvent(); event != nil; event = sdl.PollEvent() {
			switch t := event.(type) {
			case *sdl.QuitEvent:
				running = false
			case *sdl.MouseMotionEvent:
			case *sdl.MouseButtonEvent:
			case *sdl.MouseWheelEvent:
			case *sdl.KeyboardEvent:
				if t.Type == sdl.KEYDOWN {
					pressedKeys[t.Keysym.Scancode] = true
				} else {
					pressedKeys[t.Keysym.Scancode] = false
				}
			case *sdl.JoyAxisEvent:
			case *sdl.JoyBallEvent:
			case *sdl.JoyButtonEvent:
			case *sdl.JoyHatEvent:
			default:
			}

		}

		// Game Logic

		//TODO: clean this up, move it somewhere more sensible
		var moveDirection complex128

		inputArr := [5]int{sdl.SCANCODE_UP, sdl.SCANCODE_DOWN, sdl.SCANCODE_LEFT, sdl.SCANCODE_RIGHT, sdl.SCANCODE_Q}

		keyPressed := false
		for _, index := range inputArr {
			if pressedKeys[index] {
				keyPressed = true
			}
		}
		if keyPressed {
			timeDilation = (4*timeDilation + 1) / 5
		} else {
			timeDilation = (4 * timeDilation) / 5
		}

		if pressedKeys[sdl.SCANCODE_ESCAPE] {
			running = false
		}
		if pressedKeys[sdl.SCANCODE_UP] {
			moveDirection -= 0 + 1i
		}
		if pressedKeys[sdl.SCANCODE_DOWN] {
			moveDirection += 0 + 1i
		}
		if pressedKeys[sdl.SCANCODE_LEFT] {
			moveDirection -= 1 + 0i
		}
		if pressedKeys[sdl.SCANCODE_RIGHT] {
			moveDirection += 1 + 0i
		}

		hilbert.update(&theMap, &actors, moveDirection)

		for i := 0; i < len(actors); i++ {
			if actors[i].currHealth <= 0 {
				actors = append(actors[:i], actors[i+1:]...)
			}
		}

		// Rendering

		clearFrame()
		renderMap()
		presentFrame()

		// FPS limiter

		time.Sleep(time.Until(startTime.Add(DURPERFRAME)))
	}

	unloadTextures()
	sdl.Quit()
}
