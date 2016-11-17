package main

//Damage Types
const (
	dtKinetic = iota
	dtMagic   = iota
)

//Attributes
const (
	STR = iota
	DEX = iota
	CON = iota
	INT = iota
	WIS = iota
	CHA = iota
)

type Actor interface {
	update()
}

type Entity struct {
	name                  string
	maxHealth, currHealth int
	attributes            [6]int
}

type Player struct {
	this Entity
}

type NPC struct {
	id   int
	this Entity
}

func (n NPC) update(area [][]int, others []NPC) {

}

func dummyNPC() NPC {
	return NPC{id: 0, this: Entity{name: "dummy", maxHealth: 10, currHealth: 10}}
}
