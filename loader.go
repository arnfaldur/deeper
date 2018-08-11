package main

import (
	"bytes"
	"fmt"
	"github.com/veandco/go-sdl2/img"
	"io/ioutil"
	"os"
	"strconv"
	"strings"
	"time"
)

type DisplaySettings struct {
	screenWidth  int32
	screenHeight int32

	FPS      uint32
	tileSize int
	maxTiles float64
}

var loadedAtTime = make(map[string]time.Time)

func loadDisplaySettings() (DisplaySettings, bool) {
	const filepath = "settings/display.settings"

	timeLoaded, noChange := alreadyLoaded(filepath)

	if noChange {
		return DisplaySettings{}, false
	}

	file, err := ioutil.ReadFile(filepath)
	if err != nil {
		return DisplaySettings{}, false
	}

	loadedAtTime[filepath] = timeLoaded

	var ds DisplaySettings

	if DEBUGLOGGING {
		fmt.Println("Loading display settings...")
	}

	lines := getUncommentedLines(file)

	for _, l := range lines {

		tokens := strings.Split(l, " ")

		switch tokens[0] {
		case "screenwidth":
			temp, err := strconv.ParseInt(tokens[1], 10, 32)
			check(err)
			ds.screenWidth = int32(temp)
			break
		case "screenheight":
			temp, err := strconv.ParseInt(tokens[1], 10, 32)
			check(err)
			ds.screenHeight = int32(temp)
			break
		case "fps":
			temp, err := strconv.ParseInt(tokens[1], 10, 32)
			check(err)
			ds.FPS = uint32(temp)
			break
		case "tilesize":
			temp, err := strconv.ParseInt(tokens[1], 10, 32)
			check(err)
			ds.tileSize = int(temp)
			break
		case "maxtiles":
			temp, err := strconv.ParseFloat(tokens[1], 64)
			check(err)
			ds.maxTiles = temp
			break
		}
	}

	return ds, true
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

func check(e error) {
	if e != nil {
		panic(e)
	}
}

func getUncommentedLines(file []byte) []string {
	var lines []string

	//TODO make this not windows-specific
	for _, bs := range bytes.Split(file, []byte("\r\n")) {
		if len(bs) < 1 {
			continue
		}
		if bs[0] == byte('#') {
			continue
		}
		lines = append(lines, string(bs))
	}

	return lines
}

func alreadyLoaded(filepath string) (time.Time, bool) {
	info, err := os.Stat(filepath)
	if err != nil {
		return info.ModTime(), false
	}

	if val, ok := loadedAtTime[filepath]; ok {
		if val.Equal(info.ModTime()) {
			return val, true
		}
		fmt.Println("hotloaded: ", filepath)
		return info.ModTime(), false
	}
	return info.ModTime(), false
}
