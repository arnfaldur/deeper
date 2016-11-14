package main

import (
	"github.com/veandco/go-sdl2/sdl"
	"github.com/veandco/go-sdl2/sdl_gfx"
)

const (
	MAINMENU      = iota
	SCREEN_WIDTH  = 800
	SCREEN_HEIGHT = 600
	FPS           = 60
)

var fpsManager gfx.FPSmanager
var window *sdl.Window
var renderer *sdl.Renderer
var err error

func getRenderer() *sdl.Renderer {
	return renderer
}

func getWindow() *sdl.Window {
	return window
}

func initDisplay() {

	sdl.Init(sdl.INIT_EVERYTHING)

	window, err = sdl.CreateWindow("Go deeper", sdl.WINDOWPOS_UNDEFINED, sdl.WINDOWPOS_UNDEFINED,
		SCREEN_WIDTH, SCREEN_HEIGHT, sdl.WINDOW_SHOWN)
	if err != nil {
		panic(err)
	}
	//defer window.Destroy()

	gfx.InitFramerate(&fpsManager)
	gfx.SetFramerate(&fpsManager, FPS)

	renderer, err = sdl.CreateRenderer(window, -1, 0)
	if err != nil {
		panic(err)
	}

	//defer renderer.Destroy()
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
	var event sdl.Event
	for event = sdl.PollEvent(); event != nil; event = sdl.PollEvent() {
		switch event.(type) {
		case *sdl.QuitEvent:
			return false
		}
	}
	return true
}
