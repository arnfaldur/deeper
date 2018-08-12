package main

import (
	"fmt"
	"io/ioutil"
	"os"
	//"strconv"
	//"strings"
	"encoding/json"
	"github.com/veandco/go-sdl2/img"
	"path/filepath"
	"time"
)

/*

	Need to develop a monster/tile/asset cataloging system, use JSON?
		-Need to be able to:
			*add and keep track of the different kinds of monsters
			*easily add new monsters and include new fields where needed
			*


*/

type DisplaySettings struct {
	ScreenWidth  int32
	ScreenHeight int32

	FPS      uint32
	TileSize int
	MaxTiles float64
}

var loadedAtTime = make(map[string]time.Time)

func loadDisplaySettings() (DisplaySettings, bool) {
	const filepath = "settings/display.settings"

	timeLoaded, noChange := alreadyLoaded(filepath)

	if DEBUGLOGGING {
		fmt.Println("Loading display settings...")
	}

	if noChange {
		return DisplaySettings{}, false
	}

	file, err := ioutil.ReadFile(filepath)
	if err != nil {
		return DisplaySettings{}, false
	}

	loadedAtTime[filepath] = timeLoaded

	var ds DisplaySettings

	err = json.Unmarshal(file, &ds)
	if err != nil {
		return DisplaySettings{}, false
	}

	return ds, true
}

type AbstractTile struct {
	Tile
	variations int
}

var metaTiles = make(map[string]AbstractTile)

func loadTiles() {

	if len(metaTextures) == 0 {
		fmt.Fprintln(os.Stderr, "Textures must load before loading entities (Tiles)")
		os.Exit(1)
	}

	const (
		filepath = "entities/tiles.entities"
	)

	file, err := ioutil.ReadFile(filepath)
	check(err)

	var loadTiles []AbstractTile

	json.Unmarshal(file, &loadTiles)

	for i, tile := range loadTiles {
		vars := 0
		loadTiles[i].id = ID{class: TILEID, number: i}
		for _, t := range metaTextures {
			if len(t.name) >= len(tile.Name) && t.name[:len(tile.Name)] == tile.Name {
				textureID[ID{class: TILEID, number: i, state: vars}] = t.textureIndex
				vars++
			}
		}
		loadTiles[i].variations = vars
		metaTiles[tile.Name] = loadTiles[i]
	}
}

type AbstractCharacter struct {
	NPC
	variations int
}

var metaCharacters = make(map[string]AbstractCharacter)

func loadCharacters() {

	if len(metaTextures) == 0 {
		fmt.Fprintln(os.Stderr, "Textures must load before loading entities (Characters)")
		os.Exit(1)
	}

	const (
		filepath = "entities/characters.entities"
	)

	file, err := ioutil.ReadFile(filepath)
	check(err)

	var loadCharacters []AbstractCharacter

	json.Unmarshal(file, &loadCharacters)

	for i, char := range loadCharacters {
		vars := 0
		loadCharacters[i].id = ID{class: ACTORID, number: i}
		for _, t := range metaTextures {
			if len(t.name) >= len(char.Name) && t.name[:len(char.Name)] == char.Name {
				textureID[ID{class: ACTORID, number: i, state: vars}] = t.textureIndex
				vars++
			}
		}
		loadCharacters[i].variations = vars
		metaCharacters[char.Name] = loadCharacters[i]
		fmt.Printf("%+v", loadCharacters[i])
	}
}

type AbstractTexture struct {
	name         string
	path         string
	textureIndex int
}

var metaTextures []AbstractTexture

func loadTextures() {

	const (
		dir = "assets"
	)

	exts := [...]string{".png"}

	err := filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			fmt.Printf("prevent panic by handling failure accessing a path %q: %v\n", dir, err)
			return err
		}
		for _, ext := range exts {
			if path[len(path)-len(ext):] == ext {
				name := info.Name()[:len(info.Name())-len(ext)]
				metaTextures = append(metaTextures, AbstractTexture{name: name, path: path})
			}
		}
		return nil
	})

	check(err)

	for i, e := range metaTextures {
		image, err := img.Load(e.path)
		if err != nil {
			panic(err)
		}
		textures[i], err = renderer.CreateTextureFromSurface(image)
		metaTextures[i].textureIndex = i
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
