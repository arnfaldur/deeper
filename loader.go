package main

import (
	"fmt"
	"io/ioutil"
	"os"
	//"strconv"
	//"strings"
	"encoding/json"
	"github.com/veandco/go-sdl2/img"
	"github.com/veandco/go-sdl2/sdl"
	"path/filepath"
	"time"
)

type DisplaySettings struct {
	ScreenWidth  int32
	ScreenHeight int32

	FPS      uint32
	TileSize int
	MaxTiles float64
}

type AbstractTexture struct {
	name         string
	path         string
	textureIndex int
}

type AbstractTile struct {
	Tile
	variations int
}

type AbstractCharacter struct {
	NPC
	variations int
}

type AssetManager struct {
	textures  Textures
	textureID TextureAssociation

	loadedAtTime map[string]time.Time

	metaTiles      map[string]AbstractTile
	metaCharacters map[string]AbstractCharacter
	metaTextures   []AbstractTexture
}

func NewAssetManager() AssetManager {
	temp := AssetManager{}

	temp.metaTiles = make(map[string]AbstractTile)
	temp.metaCharacters = make(map[string]AbstractCharacter)
	temp.loadedAtTime = make(map[string]time.Time)
	temp.textures = make(Textures)
	temp.textureID = make(TextureAssociation)

	return temp
}

func (man *AssetManager) loadResources() {
	man.loadTextures()
	//Textures must be loaded before loading entities for file associations
	//TODO: Make this not a requirement?
	man.loadTiles()
	man.loadCharacters()
}

func (man *AssetManager) loadDisplaySettings() (DisplaySettings, bool) {
	const filepath = "settings/display.settings"

	timeLoaded, noChange := man.alreadyLoaded(filepath)

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

	man.loadedAtTime[filepath] = timeLoaded

	var ds DisplaySettings

	err = json.Unmarshal(file, &ds)
	if err != nil {
		return DisplaySettings{}, false
	}

	return ds, true
}

func (man *AssetManager) loadTiles() {

	if len(man.metaTextures) == 0 {
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
		for _, t := range man.metaTextures {
			if len(t.name) >= len(tile.Name) && t.name[:len(tile.Name)] == tile.Name {
				man.textureID[ID{class: TILEID, number: i, state: vars}] = t.textureIndex
				vars++
			}
		}
		loadTiles[i].variations = vars
		man.metaTiles[tile.Name] = loadTiles[i]
	}
}

func (man *AssetManager) loadCharacters() {

	if len(man.metaTextures) == 0 {
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
		for _, t := range man.metaTextures {
			if len(t.name) >= len(char.Name) && t.name[:len(char.Name)] == char.Name {
				man.textureID[ID{class: ACTORID, number: i, state: vars}] = t.textureIndex
				vars++
			}
		}
		loadCharacters[i].variations = vars
		man.metaCharacters[char.Name] = loadCharacters[i]
		fmt.Printf("%+v", loadCharacters[i])
	}
}

func (man *AssetManager) loadTextures() {

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
				man.metaTextures = append(man.metaTextures, AbstractTexture{name: name, path: path})
			}
		}
		return nil
	})

	check(err)

	for i, e := range man.metaTextures {
		image, err := img.Load(e.path)
		if err != nil {
			panic(err)
		}
		man.textures[i], err = renderer.CreateTextureFromSurface(image)
		man.metaTextures[i].textureIndex = i
		if err != nil {
			panic(err)
		}
		image.Free()
	}
}

func (man *AssetManager) unloadTextures() {
	for _, v := range man.textures {
		v.Destroy()
	}
}

func (man *AssetManager) getTexture(id ID) *sdl.Texture {
	return man.textures[man.textureID[id]]
}

func (man *AssetManager) alreadyLoaded(filepath string) (time.Time, bool) {
	info, err := os.Stat(filepath)
	if err != nil {
		return info.ModTime(), false
	}

	if val, ok := man.loadedAtTime[filepath]; ok {
		if val.Equal(info.ModTime()) {
			return val, true
		}
		fmt.Println("hotloaded: ", filepath)
		return info.ModTime(), false
	}
	return info.ModTime(), false
}
