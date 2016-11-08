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
	running := true
	var fpsManager gfx.FPSmanager

	window, err := sdl.CreateWindow("Go deeper", sdl.WINDOWPOS_UNDEFINED, sdl.WINDOWPOS_UNDEFINED,
		800, 600, sdl.WINDOW_SHOWN)
	if err != nil {
		panic(err)
	}
	defer window.Destroy()

	gfx.InitFramerate(&fpsManager)
	gfx.SetFramerate(&fpsManager, 60)

	surface, err := window.GetSurface()
	if err != nil {
		panic(err)
	}

	var i int32 = 0
	for running {
		running = processInputs()
		surface.FillRect(&sdl.Rect{0, 0, 800, 600}, 0xff000000)
		rect := sdl.Rect{0, 0, 200, 200}
		surface.FillRect(&rect, 0xffff0000)
		surface.FillRect(&sdl.Rect{2 * i, i, 400, 400}, 0xff00ff00)
		window.UpdateSurface()
		i++
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
