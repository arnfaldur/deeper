package main

import (
	"fmt"
	"math/cmplx"
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
	SPD
)

type Actor interface {
	process()
}

type Entity struct {
	pos  complex128
	name string
}

type Character struct {
	Entity
	maxHealth, currHealth int
	attributes            [7]int
	damage                int
	vel                   complex128
	size                  float64
}

type Player struct {
	Character
	id ID
}

type NPC struct {
	Character
	id ID
}

func (n NPC) update() {

}

func (n NPC) isAtPos(xpos, ypos int) bool {
	return (int(real(n.pos)) == xpos) && (int(imag(n.pos)) == ypos)
}

func (p *Player) termupdate(floor *Mapt, others *[]NPC, action int) {
	xpos := int(real(p.pos))
	ypos := int(imag(p.pos))

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

	if DEBUGLOGGING {
		fmt.Print("Action: ")
		switch action {
		case UP:
			fmt.Println("UP")
		case LEFT:
			fmt.Println("LEFT")
		case DOWN:
			fmt.Println("DOWN")
		case RIGHT:
			fmt.Println("RIGHT")
		}
		fmt.Println("xpos", xpos)
		fmt.Println("ypos", ypos)
	}

	p.move(xpos, ypos, floor, others)
}

func (a *Character) attack(v *Character) {
	v.currHealth -= a.damage
}

func (p *Character) move(xpos, ypos int, floor *Mapt, others *[]NPC) {

	var t = (*floor)[ypos][xpos]
	if DEBUGLOGGING {
		fmt.Println("IS SOLID", IS_SOLID[t.tileID])
	}
	if !IS_SOLID[t.tileID] {

		for i := 0; i < len(*others); i++ {
			if (*others)[i].isAtPos(xpos, ypos) {
				//p.attack(&(*others)[i])
				break
			}
		}
		p.pos = complex(float64(xpos), float64(ypos))
		if DEBUGLOGGING {
			fmt.Println("xpos,ypos: ", xpos, ypos)
		}
	}
}

func (p *Player) update(theMap *Mapt, actors *[]NPC, moveDirection complex128) {
	if cmplx.Abs(moveDirection) > 1 {
		moveDirection = moveDirection / complex(cmplx.Abs(moveDirection), 0)
	}
	p.vel = approach(p.vel, moveDirection/5)
	newPos := p.pos + p.vel

	px := real(newPos)
	py := imag(newPos)

	pxf, pxc := int(px), int(px+1)
	pyf, pyc := int(py), int(py+1)

	for y := pyf; y < pyc; y++ {
		for x := pxf; x < pxc; x++ {
			if IS_SOLID[theMap[y][x].tileID] {

			}
		}
	}

	p.pos = newPos
}

func testEnemyNPC(pos complex128, id int) NPC {
	return NPC{Character{Entity: Entity{pos: pos, name: "TestEnemy"}, size: 0.8, maxHealth: 10, currHealth: 10}, makeActorID(id)}
}

func dummyNPC(x, y int) NPC {
	return NPC{Character{Entity: Entity{pos: complex(float64(x), float64(y)), name: "dummy"}, maxHealth: 10, currHealth: 10}, DUMMY}
}

func approach(vel, target complex128) complex128 {
	return (vel*4 + target) / 5
}
