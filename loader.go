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
	check(err)

	if val, ok := loadedAtTime[filepath]; ok {
		if val.Equal(info.ModTime()) {
			return val, true
		}
		fmt.Println("hotloaded: ", filepath)
		return info.ModTime(), false
	}
	return info.ModTime(), false
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
