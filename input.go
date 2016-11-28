package main

import (
	"fmt"
	"github.com/veandco/go-sdl2/sdl"
)

var (
	INPUT_INIT bool = false
)

var (
	KPR = make([]uint8, 512, 512)
	KDN = make([]uint8, 512, 512)
	KUP = make([]uint8, 512, 512)
)

func init_key_state() {
	KPR = sdl.GetKeyboardState()
	KDN = KPR
	KUP = KPR

	fmt.Println("Input initialized: KPR, KDN, KUP", len(KPR), len(KDN), len(KUP))
	INPUT_INIT = true
}

func get_key_state(key int) bool {
	sdl.PumpEvents()
	temp := sdl.GetKeyboardState()
	//temp2 := KPR
	copy(KPR, temp)
	return temp[key] != 0 //&& temp2[key] == 0
}

func update_key_state() {

	/*if !INPUT_INIT {
		fmt.Println("input was not initialized!")
		//init_key_state()
		return
	}*/

	sdl.PumpEvents()
	temp := sdl.GetKeyboardState()

	for i := 0; i < len(temp); i++ {

		if temp[i] == 1 {
			fmt.Println("PRESSED: ", i)
		}

		if KPR[i] == 0 && temp[i] == 1 {
			KDN[i] = 1
		} else {
			KDN[i] = 0
		}

		if KPR[i] == 1 && temp[i] == 0 {
			KUP[i] = 1
		} else {
			KUP[i] = 0
		}

		//KPR[i] = temp[i]
	}

	//fmt.Println(temp)
	//fmt.Println(KUP)
	//fmt.Println(KDN)
}

func processInputs() bool {
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

	fmt.Println("I HAVE PROCESSED INPUTS!")

	return true
}
