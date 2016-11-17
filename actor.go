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

type NPC struct {
	id                    int
	name                  string
	maxHealth, currHealth int
	attributes            [6]int
}

func dummyNPC() NPC {
	return NPC{id: 0, name: "dummy", maxHealth: 10, currhealth: 10}
}
