package main

import (
	"fmt"
)

//Direction
const (
	UP = iota
	DOWN
	LEFT
	RIGHT
)

//Damage Types
const (
	dtKinetic = iota
	dtMagic
)

//Attributes
const (
	STR = iota
	DEX
	CON
	INT
	WIS
	CHA
)

type Actor interface {
	process()
}

type Entity struct {
	x, y                  int
	name                  string
	maxHealth, currHealth int
	attributes            [6]int
	damage                int
}

type Player struct {
	Entity
	id ID
}

type NPC struct {
	Entity
	id ID
}

func (n NPC) update() {

}

func (n NPC) isAtPos(xpos, ypos int) bool {
	return (n.x == xpos) && (n.y == ypos)
}

func (p *Player) termupdate(floor *Mapt, others *[]NPC, action int) {
	xpos := p.x
	ypos := p.y

	fmt.Println("Action: ", action)
	fmt.Println("UP: ", UP)
	fmt.Println("DOWN: ", DOWN)
	fmt.Println("LEFT: ", LEFT)
	fmt.Println("RIGHT: ", RIGHT)

	switch action {
	case UP:
		ypos -= 1
	case LEFT:
		xpos -= 1
	case DOWN:
		ypos += 1
	case RIGHT:
		xpos += 1
	}

	fmt.Println("xpos", xpos)
	fmt.Println("ypos", ypos)

	p.move(xpos, ypos, floor, others)
}

func (p *Player) attack(n *NPC) {
	n.currHealth -= p.damage
}

func (p *Player) move(xpos int, ypos int, floor *Mapt, others *[]NPC) {

	var t = (*floor)[ypos][xpos]

	fmt.Println("IS SOLID", IS_SOLID[t.tileID])

	if !IS_SOLID[t.tileID] {

		for i := 0; i < len(*others); i++ {
			if (*others)[i].isAtPos(xpos, ypos) {
				p.attack(&(*others)[i])
				return
			}
		}

		p.x = xpos
		p.y = ypos

		fmt.Println("xpos,ypos: ", xpos, ypos)
	}
}

func (p *Player) update() {

}

func testEnemyNPC(xpos, ypos, id int) NPC {
	return NPC{Entity{x: xpos, y: ypos, name: "TestEnemy", maxHealth: 10, currHealth: 10}, makeActorID(id)}
}

func dummyNPC(xpos, ypos int) NPC {
	return NPC{Entity{x: xpos, y: ypos, name: "dummy", maxHealth: 10, currHealth: 10}, DUMMY}
}
