package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/sdl"
	"github.com/veandco/go-sdl2/sdl_gfx"
	"github.com/veandco/go-sdl2/sdl_image"
	"github.com/veandco/go-sdl2/sdl_ttf"
	"os"
)

type Point struct {
	x int
	y int
}

type Textures map[int]*sdl.Texture

const (
	MAINMENU = iota
)

var (
	SCREEN_WIDTH  = 400
	SCREEN_HEIGHT = 300
)

const (
	FPS       = 60
	TILE_SIZE = 64
)

var fpsManager gfx.FPSmanager

// var font *ttf.Font
var window *sdl.Window
var renderer *sdl.Renderer
var texture *sdl.Texture
var err error
var textures Textures = make(Textures)

func getRenderer() *sdl.Renderer {
	return renderer
}

func getWindow() *sdl.Window {
	return window
}

func initDisplay() error {

	sdl.Init(sdl.INIT_EVERYTHING)

	if err := ttf.Init(); err != nil {
		fmt.Fprintf(os.Stderr, "Failed to initialize TTF: %s\n", err)
	}

	window, err = sdl.CreateWindow("Go deeper", sdl.WINDOWPOS_UNDEFINED, sdl.WINDOWPOS_UNDEFINED,
		SCREEN_WIDTH, SCREEN_HEIGHT, sdl.WINDOW_SHOWN)
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
	gfx.SetFramerate(&fpsManager, FPS)

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

func processInputs() bool {

	SCREEN_WIDTH, SCREEN_HEIGHT = window.GetSize()

	var event sdl.Event
	for event = sdl.PollEvent(); event != nil; event = sdl.PollEvent() {
		switch event.(type) {
		//switch t := event.(type) {
		case *sdl.QuitEvent:
			return false
		case *sdl.MouseButtonEvent:
		case *sdl.KeyDownEvent:
		case *sdl.KeyUpEvent:

		}
	}
	return true
}

func renderGame() {
	xoffset := ((SCREEN_WIDTH % (TILE_SIZE * 2)) - TILE_SIZE) / 2
	yoffset := ((SCREEN_HEIGHT % (TILE_SIZE * 2)) - TILE_SIZE) / 2
	for i := 0; i < SCREEN_HEIGHT/TILE_SIZE+1; i++ {
		for j := 0; j < SCREEN_WIDTH/TILE_SIZE+1; j++ {
			drawTile(textures[0], j*TILE_SIZE+xoffset, i*TILE_SIZE+yoffset)
		}
	}
}

func drawTile(tile *sdl.Texture, x, y int) {
	src := sdl.Rect{0, 0, 64, 64}
	dst := sdl.Rect{int32(x), int32(y), 64, 64}
	renderer.Copy(tile, &src, &dst)

}

func loadTextures() {
	assets := []string{"./assets/ShittyTile.png"}
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
