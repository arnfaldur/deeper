package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/gfx"
	"github.com/veandco/go-sdl2/sdl"
	"github.com/veandco/go-sdl2/ttf"
	"math"
	"os"
	"runtime"
)

type Textures map[int]*sdl.Texture

var (
	windowWidth  int32
	windowHeight int32
)

var ds DisplaySettings

var fpsManager gfx.FPSmanager

// var font *ttf.Font
var window *sdl.Window
var renderer *sdl.Renderer
var texture *sdl.Texture
var err error
var textures = make(Textures)

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

func renderMap() {

	windowWidth, windowHeight = window.GetSize()

	//Dirty hot-loading
	temp, ok := loadDisplaySettings()
	if ok {
		ds = temp
	}

	px, py := parts((hilbert).pos)

	tilesToTop := ds.maxTiles / 2
	tilesToSide := tilesToTop / float64(windowHeight) * float64(windowWidth)

	for i := int(py - tilesToTop); i <= int(py+1+tilesToTop); i++ {
		for j := int(px - tilesToSide); j <= int(px+1+tilesToSide); j++ {

			if i >= 0 && i < len(theMap) && j >= 0 && j < len((theMap)[0]) {
				tile := theMap[i][j]
				drawTile(textures[tile.id.textureID], tile.pos)
			}
		}
	}

	for _, npc := range actors {
		if real(npc.pos) <= px+tilesToSide+1 && imag(npc.pos) <= py+tilesToTop+1 {
			drawTile(textures[npc.id.textureID+3], npc.pos)
		}
	}

	//draws hilbert
	drawTile(textures[hilbert.id.textureID], hilbert.pos)
}

func drawTile(texture *sdl.Texture, pos complex128) {
	scale := math.Floor(float64(windowWidth) / ds.maxTiles)

	//Center coordinate system on hilbert's center
	pos -= hilbert.pos + complex(0.5, 0.5)
	pos = cmul(pos, scale)

	x, y := parts(pos)

	//source rectangle of texture, should currently be the same size as the picture
	src := sdl.Rect{W: int32(ds.tileSize), H: int32(ds.tileSize)}
	//Destination rectangle, scaled so that x and y are integers from 0 - 16
	dst := sdl.Rect{
		X: int32(x + float64(windowWidth)/2),
		Y: int32(y + float64(windowHeight)/2),
		W: int32(scale),
		H: int32(scale),
	}
	//Draw tile to the renderer
	renderer.Copy(texture, &src, &dst)
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
