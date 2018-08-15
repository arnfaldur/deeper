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
	MAXCHARSIZE float64       = 0.8
)

type Mapt [MAPSIZE][MAPSIZE]Tile

var theMap Mapt
var hilbert Player
var actors []NPC
var environment []*Tile
var AssMan AssetManager

var timeDilation = 0.0

func populateMap() {
	//fmt.Printf("STONE_FLOOR: %v\n", STONE_FLOOR)
	for y := 0; y < MAPSIZE; y++ {
		for x := 0; x < MAPSIZE; x++ {
			//true at edges and random points, for flavour, RNG is deterministic unless seeded.
			randomN := rand.Float64()
			if y == 0 || x == 0 || y == MAPSIZE-1 || x == MAPSIZE-1 || randomN > 0.8 {
				theMap[y][x] = NewTile("STONE_WALL", float64(x), float64(y))
				environment = append(environment, &theMap[y][x])
			} else {
				if randomN > 0.6 {
					actors = append(actors, NewNPC("TestEnemy", float64(x), float64(y)))
				}
				theMap[y][x] = NewTile("STONE_FLOOR", float64(x), float64(y))
				environment = append(environment, &theMap[y][x])
			}
		}
	}
}

func main() {

	AssMan = NewAssetManager()

	//Requires AssMan
	initDisplay()
	defer destroyDisplay()

	AssMan.loadResources()

	running := true
	var event sdl.Event
	var pressedKeys [512]bool

	hilbert = Player{Character{Entity: Entity{id: ID{PLAYERID, 0, 0}, pos: 3 + 3i, Size: 0.8}, damage: 5}}

	//fucking awful
	//TODO: fix this garbage
	for _, texture := range AssMan.metaTextures {
		if texture.name == "PLAYER" {
			AssMan.textureID[hilbert.id] = texture.textureIndex
		}
	}

	populateMap()
	var interactions []Interaction

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

		inputArr := [...]int{sdl.SCANCODE_UP, sdl.SCANCODE_DOWN, sdl.SCANCODE_LEFT, sdl.SCANCODE_RIGHT, sdl.SCANCODE_Q}

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

		theMap.locateNPCs(actors)

		findAllCollisions(theMap, hilbert, actors, interactions)
		dealWithCollisions(&theMap, &hilbert, actors, moveDirection)

		for i := 0; i < len(actors); i++ {
			if actors[i].CurrHealth <= 0 {
				actors = append(actors[:i], actors[i+1:]...)
			}
		}
		theMap.locateNPCs(actors)

		// Rendering

		clearFrame()
		renderMap()

		for _, t := range vicinity(hilbert.pos, (hilbert.Size+MAXCHARSIZE+sqrt2)/2) {
			if t[0] >= 0 && t[0] < MAPSIZE && t[1] >= 0 && t[1] < MAPSIZE {
				for i := range theMap[t[0]][t[1]].npcsOnTile {
					drawTile(AssMan.textures[16], theMap[t[0]][t[1]].npcsOnTile[i].pos)
				}
			}
		}
		//for _, e := range vicinity(hilbert.pos, hilbert.size+MAXCHARSIZE+1.5) {
		//	if e[0] >= 0 && e[0] < MAPSIZE && e[1] >= 0 && e[1] < MAPSIZE {
		//		drawTile(textures[16], complex(float64(e[0]), float64(e[1])))
		//	}
		//}
		presentFrame()

		// FPS limiter

		//gfx.FramerateDelay(&fpsManager)
		time.Sleep(time.Until(startTime.Add(DURPERFRAME)))
	}

	AssMan.unloadTextures()
	sdl.Quit()
}
