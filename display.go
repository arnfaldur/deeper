package main

import (
	"github.com/veandco/go-sdl2/sdl"
	"github.com/veandco/go-sdl2/sdl_gfx"
)

const (
	MAINMENU = iota
)

func startRenderLoop() {
	sdl.Init(sdl.INIT_EVERYTHING)
	//	gamestate := MAINMENU
	var fpsManager gfx.FPSmanager
	var window *sdl.Window
	var renderer *sdl.Renderer
	var err error

	window, err = sdl.CreateWindow("Go deeper", sdl.WINDOWPOS_UNDEFINED, sdl.WINDOWPOS_UNDEFINED,
		800, 600, sdl.WINDOW_SHOWN)
	if err != nil {
		panic(err)
	}
	defer window.Destroy()

	gfx.InitFramerate(&fpsManager)
	gfx.SetFramerate(&fpsManager, 60)

	// surface, err := window.GetSurface()
	renderer, err = sdl.CreateRenderer(window, -1, 0)
	if err != nil {
		panic(err)
	}
	defer renderer.Destroy()

	running := true
	for running {
		running = processInputs()
		rect := sdl.Rect{0, 0, 200, 200}

		renderer.Clear()
		renderer.SetDrawColor(0,0,0,0)
		renderer.FillRect(&sdl.Rect{0,0,800,600})
		renderer.SetDrawColor(0xff,0,0,0xff)
		renderer.DrawRect(&rect)
		renderer.Present()

		gfx.FramerateDelay(&fpsManager)
	}

	sdl.Quit()
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
