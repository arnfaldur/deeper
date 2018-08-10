package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/gfx"
	"github.com/veandco/go-sdl2/img"
	"github.com/veandco/go-sdl2/sdl"
	"github.com/veandco/go-sdl2/ttf"
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

var (
	SCREEN_WIDTH  int32 = 800
	SCREEN_HEIGHT int32 = 600
)

const (
	FPS       = 60
	TILE_SIZE = 1 << 5
	MAX_TILES = 1 << 4 // 16 supreme
)

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

	sdl.Init(sdl.INIT_EVERYTHING)

	//Demon magic that fixes unresponsive bug on OS X
	runtime.LockOSThread()

	if err := ttf.Init(); err != nil {
		fmt.Fprintf(os.Stderr, "Failed to initialize TTF: %s\n", err)
	}

	window, err = sdl.CreateWindow("Go deeper", sdl.WINDOWPOS_UNDEFINED, sdl.WINDOWPOS_UNDEFINED,
		SCREEN_WIDTH, SCREEN_HEIGHT, sdl.WINDOW_SHOWN|sdl.WINDOW_RESIZABLE)
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

func renderMap(themap *Mapt, actors *[]NPC, hilbert *Player) {

	SCREEN_WIDTH, SCREEN_HEIGHT = window.GetSize()

	px := real((*hilbert).pos)
	py := imag((*hilbert).pos)

	for i := int(py) - MAX_TILES/2; i <= int(py+MAX_TILES/2+2); i++ {
		for j := int(px) - MAX_TILES/2; j <= int(px+MAX_TILES/2+2); j++ {

			if i >= 0 && i < len(*themap) && j >= 0 && j < len((*themap)[0]) {
				drawTile(textures[(*themap)[i][j].tileID.number], float64(j)-px+MAX_TILES/2-1, float64(i)-py+MAX_TILES/2-1)
			}
		}
	}
	for i := 0; i < len(*actors); i++ {
		if (real((*actors)[i].pos)) <= px+MAX_TILES/2+2 && (imag((*actors)[i].pos)) <= py+MAX_TILES/2+2 {
			drawTile(textures[(*actors)[i].id.number+3], real((*actors)[i].pos)-px+(MAX_TILES/2-1), imag((*actors)[i].pos)-py+(MAX_TILES/2-1))
		}
	}
	drawTile(textures[2], MAX_TILES/2-0.5, MAX_TILES/2-0.5)
}

func drawTile(texture *sdl.Texture, x, y float64) {
	scale := float64(SCREEN_HEIGHT / MAX_TILES)

	//source rectangle of texture, should currently be the same size as the picture
	src := sdl.Rect{W: int32(TILE_SIZE), H: int32(TILE_SIZE)}
	//Destination rectangle, scaled so that x and y are integers from 0 - 16
	dst := sdl.Rect{X: int32(x * scale), Y: int32(y * scale), W: int32(scale), H: int32(scale)}
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
		"assets/ShittyBeholder.png"}
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
