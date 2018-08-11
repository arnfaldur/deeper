package main

import (
	"bytes"
	"fmt"
	"io/ioutil"
	"os"
	"strconv"
	"strings"
	"time"
)

var loadedAtTime = make(map[string]time.Time)

type Tester struct {
	name     string
	strength float64
	versions int64
	dir      complex128
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

func loadDisplaySettings() (DisplaySettings, bool) {
	const filepath = "settings/display.settings"

	timeLoaded, loaded := alreadyLoaded(filepath)

	if loaded {
		return DisplaySettings{}, false
	}

	file, err := ioutil.ReadFile(filepath)
	if err != nil {
		return DisplaySettings{}, false
	}

	loadedAtTime[filepath] = timeLoaded

	var ds DisplaySettings

	fmt.Println("Loading display settings...")

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

func loadTesters() {

	const filepath = "settings/test.settings"

	timeLoaded, loaded := alreadyLoaded(filepath)

	if loaded {
		return
	}

	file, err := ioutil.ReadFile(filepath)
	if err != nil {
		return
	}

	loadedAtTime[filepath] = timeLoaded

	var testers []Tester

	fmt.Println("Loading testers")

	lines := getUncommentedLines(file)

	for _, l := range lines {

		tokens := strings.Split(l, " ")

		t := Tester{name: tokens[0]}
		tokens = tokens[1:]

		for i := 0; i < len(tokens); i++ {
			switch tokens[i] {
			case "strength":
				t.strength, err = strconv.ParseFloat(tokens[i+1], 64)
				check(err)
				i += 1
				break
			case "versions":
				t.versions, err = strconv.ParseInt(tokens[i+1], 10, 64)
				i += 1
				break
			case "dir":
				dirreal, err := strconv.ParseFloat(tokens[i+1], 64)
				check(err)
				dirimag, err := strconv.ParseFloat(tokens[i+2], 64)
				check(err)
				t.dir = complex(dirreal, dirimag)
				i += 2
				break
			}
		}

		testers = append(testers, t)
	}

	for _, tester := range testers {
		fmt.Printf("%+v\n", tester)
	}
	//return testers
}
