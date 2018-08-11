package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/gfx"
	"github.com/veandco/go-sdl2/img"
	"github.com/veandco/go-sdl2/sdl"
	"github.com/veandco/go-sdl2/ttf"
	"math"
	"os"
	"runtime"
)

type Point struct {
	x int
	y int
}

type Textures map[int]*sdl.Texture

const (
	MAINMENU = iota
)

type DisplaySettings struct {
	screenWidth  int32
	screenHeight int32

	FPS      uint32
	tileSize int
	maxTiles float64
}

var (
	WINDOW_WIDTH  int32
	WINDOW_HEIGHT int32
)

var ds DisplaySettings

var fpsManager gfx.FPSmanager

// var font *ttf.Font
var window *sdl.Window
var renderer *sdl.Renderer
var texture *sdl.Texture
var err error
var textures = make(Textures)

func getRenderer() *sdl.Renderer {
	return renderer
}

func getWindow() *sdl.Window {
	return window
}

func initDisplay() error {

	temp, ok := loadDisplaySettings()

	if ok {
		ds = temp
	} else {
		fmt.Println("Display initialization failed: could not load display settings")
	}

	sdl.Init(sdl.INIT_EVERYTHING)

	//Demon magic that fixes unresponsive bug on OS X
	runtime.LockOSThread()

	if err := ttf.Init(); err != nil {
		fmt.Fprintf(os.Stderr, "Failed to initialize TTF: %s\n", err)
	}

	window, err = sdl.CreateWindow("Go deeper", sdl.WINDOWPOS_UNDEFINED, sdl.WINDOWPOS_UNDEFINED,
		ds.screenWidth, ds.screenHeight, sdl.WINDOW_SHOWN|sdl.WINDOW_RESIZABLE)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to create window: %s\n", err)
		return err
	}
	//defer window.Destroy()

	// if font, err = ttf.OpenFont("../../assets/test.ttf", 32); err != nil {
	// 	fmt.Fprint(os.Stderr, "Failed to open font: %s\n", err)
	// 	return err
	// }

	// defer font.Close()

	// if surface, err = window.GetSurface(); err != nil {
	// 	fmt.Fprint(os.Stderr, "Failed to get window surface: %s\n", err)
	// 	return err
	//}

	gfx.InitFramerate(&fpsManager)
	gfx.SetFramerate(&fpsManager, ds.FPS)

	renderer, err = sdl.CreateRenderer(window, -1, 0)
	if err != nil {
		return err
	}

	//defer renderer.Destroy()
	return nil
}

func destroyDisplay() {
	window.Destroy()
	renderer.Destroy()
}

func clearFrame() {
	renderer.Clear()
}

func presentFrame() {
	renderer.Present()
	gfx.FramerateDelay(&fpsManager)
}

func renderMap(theMap *Mapt, actors *[]NPC, hilbert *Player) {

	WINDOW_WIDTH, WINDOW_HEIGHT = window.GetSize()

	//Dirty hotloading
	temp, ok := loadDisplaySettings()
	if ok {
		ds = temp
	}

	px, py := parts((*hilbert).pos)

	tilesToTop := ds.maxTiles / 2
	tilesToSide := tilesToTop / float64(WINDOW_HEIGHT) * float64(WINDOW_WIDTH)

	for i := int(py - tilesToTop); i <= int(py+1+tilesToTop); i++ {
		for j := int(px - tilesToSide); j <= int(px+1+tilesToSide); j++ {

			if i >= 0 && i < len(*theMap) && j >= 0 && j < len((*theMap)[0]) {
				drawTile(textures[(*theMap)[i][j].tileID.number], float64(j)-px, float64(i)-py)
			}
		}
	}
	for _, npc := range *actors {
		if real(npc.pos) <= px+tilesToSide+1 && imag(npc.pos) <= py+tilesToTop+1 {
			drawTile(textures[npc.id.number+3], real(npc.pos)-px, imag(npc.pos)-py)
		}
	}
	drawTile(textures[2], 0, 0)
}

func drawTile(texture *sdl.Texture, x, y float64) {
	scale := math.Floor(float64(WINDOW_HEIGHT) / ds.maxTiles)

	//source rectangle of texture, should currently be the same size as the picture
	src := sdl.Rect{W: int32(ds.tileSize), H: int32(ds.tileSize)}
	//Destination rectangle, scaled so that x and y are integers from 0 - 16
	dst := sdl.Rect{X: int32(float64(WINDOW_WIDTH)/2 + (x-0.5)*scale), Y: int32(float64(WINDOW_HEIGHT)/2 + (y-0.5)*scale), W: int32(scale), H: int32(scale)}
	//Draw tile to the renderer
	renderer.Copy(texture, &src, &dst)

}

func loadTextures() {
	assets := []string{
		"assets/STONE_WALL.png",
		"assets/STONE_FLOOR.png",
		"assets/PLAYER.png",
		"assets/enemies/TestEnemy0.png",
		"assets/enemies/TestEnemy1.png",
		"assets/enemies/TestEnemy2.png",
		"assets/enemies/TestEnemy3.png",
		"assets/enemies/TestEnemy4.png",
		"assets/enemies/TestEnemy5.png",
		"assets/enemies/TestEnemy6.png",
		"assets/enemies/TestEnemy7.png",
		"assets/enemies/TestEnemy8.png",
		"assets/enemies/TestEnemy9.png",
		"assets/ShittyTile.png",
		"assets/ShittyGuy.png",
		"assets/ShittyBeholder.png",
		"assets/STONE_WALL_RED.png",
	}
	for i, e := range assets {
		image, err := img.Load(e)
		if err != nil {
			panic(err)
		}
		textures[i], err = renderer.CreateTextureFromSurface(image)
		if err != nil {
			panic(err)
		}
		image.Free()
	}
}

func unloadTextures() {
	for _, v := range textures {
		v.Destroy()
	}
}
